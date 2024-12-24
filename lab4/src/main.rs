use bson::doc;
use bson::oid::ObjectId;
use mongodb::{Client, Database, Collection};
use rocket::futures::channel::mpsc::{channel, Sender};
use rocket::futures::future::join_all;
use rocket::tokio::select;
use rocket::tokio::sync::Mutex;
use rocket::{futures::TryStreamExt, http::CookieJar};
use rocket::form::Form;
use rocket::http::Cookie;
use rocket::serde::json::{Json, from_str, to_string};
use serde::{Serialize, Deserialize};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use uuid_by_string::uuid;
use ws;
use std::sync::Arc;
#[macro_use] extern crate rocket;

type Connections = Arc<Mutex<HashMap<String, Sender<String>>>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    password: String,
    session: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CuttedUser {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
}

#[derive(FromForm)]
struct AuthUser <'r>{
    name: &'r str,
    password: &'r str
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct  Message {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    content: String,
    user: CuttedUser
}

#[derive(Serialize)]
struct StatusInfo {
    error: bool,
    detail: String,
    messages: Option<Vec<Message>>,
    users: Option<Vec<CuttedUser>>
}

async fn create_mongo() -> (Client, Database) {
    let mongo_uri = env::var("MONGO_URI").expect("Not MONGO_URI");
    let mongo_db = env::var("MONGO_DB").expect("NOT MONGO_DB");

    let client = Client::with_uri_str(&mongo_uri).await.expect("Cannot get client");
    let database = client.database(&mongo_db);

    (client, database)
}

#[get("/ws/<user>")]
fn echo_channel(
    user: String, 
    ws: ws::WebSocket, 
    connections: &rocket::State<Connections>
) -> ws::Channel<'static> {
    use rocket::futures::{SinkExt, StreamExt};
    let connections = connections.inner().clone();

    ws.channel(move |mut stream| {
        Box::pin(async move{
            let (tx, mut rx) = channel(1);
            connections.lock().await.insert(user.clone(), tx);
            
            loop {
                select! {
                    message = stream.next() => match message {
                        Some(Ok(ws::Message::Text(res))) => {
                            match from_str::<Message>(&res) {
                                Ok(msg) => {
                                    let serialized = bson::to_bson(&msg.clone()).unwrap();
                                    let document = serialized.as_document().unwrap();
                                    let serialized_c = to_string::<Message>(&msg.clone()).unwrap();
                                    let (_, database) = create_mongo().await;

                                    _ = database
                                        .collection("messages")
                                        .insert_one(document.to_owned())
                                        .await;
                                    _ = stream.send(ws::Message::from(serialized_c.clone())).await;
        
                                    let mut locked = connections.lock().await;
                                    let res = locked.iter_mut().filter_map(|(peer, tx)| {
                                        if peer != &user {
                                            Some(tx.send(serialized_c.clone()))
                                        } else {
                                            None
                                        }
                                    }).collect::<Vec<_>>();
                                    _ = join_all(res).await;
                                },
                                Err(_) => {},
                            };
                        },
                        Some(Ok(ws::Message::Close(_))) => {
                            connections.lock().await.remove(&user);
                            break;
                        },
                        Some(Err(_)) => {
                            connections.lock().await.remove(&user);
                            break;
                        },
                        None => break,
                        _ => {},
                    },
                    message = rx.next() => {
                        if let Some(message) = message {
                            // let serialized = bson::to_bson(&message.clone()).unwrap();
                            // let document = serialized.as_document().unwrap();
                            // let serialized_c = from_str::<Message>(&message.clone()).unwrap();
                            // let serialized_c = to_string::<Message>(&message.clone()).unwrap();
                            let _ = stream.send(ws::Message::from(message.clone())).await;
                        }
                    },
                    else => break,
                }
            }
            Ok(())
        })
    })
}

#[post("/signin", data = "<user>")]
async fn signin(
    user: Form<AuthUser<'_>>,
    cookies: &CookieJar<'_>
) -> Json<StatusInfo> {
    let name = String::from(user.name);
    let session_value = uuid::generate(name.as_str(), None).unwrap();
    let user_doc = User {
        id: None,
        name: String::from(user.name),
        password: String::from(user.password),
        session: Some(session_value.clone())
    };
    let serialized = bson::to_bson(&user_doc).unwrap();
    let document = serialized.as_document().unwrap();

    let (_, database) = create_mongo().await;
    let res = database
        .collection("users")
        .insert_one(document.to_owned())
        .await;

    if res.is_err() {
        return Json(StatusInfo{
            error: true,
            detail: res.err().unwrap().to_string(),
            messages: None,
            users: None
        });
    }

    let cookie = Cookie::new("session", session_value);
    cookies.add(cookie);

    let user_id = res.unwrap().inserted_id.as_object_id();
    Json(StatusInfo{
        error: false,
        detail: "user was signed".to_string(),
        messages: None,
        users: Some(vec![CuttedUser{
            id: user_id,
            name: name
        }])
    })
}

#[post("/auth", data = "<user>")]
async fn auth(
    user: Form<AuthUser<'_>>,
    cookies: &CookieJar<'_>
) -> Json<StatusInfo> {
    
    let name = String::from(user.name);
    let password = String::from(user.password);
    let filetr = doc! {
        "name": name.as_str().to_owned(), 
        "password": password.as_str().to_owned()
    };

    let (_, database) = create_mongo().await;
    let collection: Collection<User> = database.collection("users");

    let res = collection
    .find_one(filetr)
    .await;

    let opt_user = res.unwrap();
    match opt_user {
        Some(user) => {
            let session_value = uuid::generate(
                &user.name.clone(), None
            ).unwrap();

            let cookie = Cookie::new("session", session_value.clone());
            cookies.add(cookie);

            let res = collection
                .update_one(
                    doc! {"_id": user.id.to_owned()},
                    doc! {"$set": {"session": session_value.to_owned()}}
                )
                .await;

            if res.is_err() {
                return Json(StatusInfo{
                    error: true,
                    detail: res.err().unwrap().to_string(),
                    messages: None,
                    users: None
                });
            }

            Json(StatusInfo{
                error: false,
                detail: "user was authed".to_string(),
                messages: None,
                users: Some(vec![CuttedUser{
                    id: user.id.clone(),
                    name: user.name.clone()
                }])
            })
        },
        None => {
            Json(StatusInfo{
                error: true,
                detail: "cannot find user".to_string(),
                messages: None,
                users: None
            })
        },
    }
}

#[post["/quit"]]
async fn quit(
    cookies: &CookieJar<'_>
) -> Json<StatusInfo> {
    let session = cookies.get("session");

    match session {
        Some(session) => {
            let (_, database) = create_mongo().await;
            let collection: Collection<User> = database.collection("users");

            let res = collection.update_one(
                doc! {"session": session.value().to_owned()},
                doc! {"$set": {"session": "".to_owned()}}
            ).await;

            if res.is_err() {
                return Json(StatusInfo{
                    error: true,
                    detail: res.err().unwrap().to_string(),
                    messages: None,
                    users: None
                });
            }

            cookies.remove("session");

            Json(StatusInfo{
                error: false,
                detail: "user was quited".to_string(),
                messages: None,
                users: None
            })
        },
        None => {
            Json(StatusInfo{
                error: true,
                detail: "cannot get session".to_string(),
                messages: None,
                users: None
            })
        },
    }
}

#[get("/messages?<take>&<skip>")]
async fn get_messages(
    cookies: &CookieJar<'_>,
    skip: Option<u8>,
    take: Option<u8>,
) -> Json<StatusInfo> {
    let session = cookies
        .get("session")
        .expect("Unauthed")
        .value();
    
    let (_, database) = create_mongo().await;
    let user_collection: Collection<User> = database.collection("users");

    let res = user_collection
        .find_one(doc! {"session": session.to_owned()})
        .await.
        unwrap();

    if res.is_none() {
        // return Messages{messages: vec![]};
        return Json(StatusInfo{
            error: true,
            detail: "cannot find such user".to_string(),
            messages: None,
            users: None
        });
    }

    let skip = match skip {
        Some(skip) => skip,
        None => 0,
    };
    let take = match take {
        Some(take) => take,
        None => 10,
    };

    let message_collection: Collection<Message> = database.collection("messages");
    let res = message_collection
        .find(doc! {})
        .sort(doc! {"_id": -1})
        .skip(skip as u64)
        .limit(take as i64)
        .await;

    match res {
        Ok(mut cur) => {
            let mut messages: Vec<Message> = Vec::new();
            let mut status = true;

            while status {
                let msg_cur = cur.try_next().await;
                
                if msg_cur.is_ok() {
                    status = match msg_cur.unwrap() {
                        Some(message) => {
                            messages.push(message);
                            true
                        },
                        None => {false},
                    }
                } else {
                    status = false;
                }
            }
            messages.reverse();

            Json(StatusInfo{
                error: false,
                detail: "messages!".to_string(),
                messages: Some(messages),
                users: None
            })
        },
        Err(err) => {
            Json(StatusInfo{
                error: true,
                detail: err.to_string(),
                messages: None,
                users: None
            })
        },
    }
}

// #[get("/files/<file_id>")]
// fn get_file(file_id: String) {

// }

// #[post("/files?")]
// fn upload_file() {
    
// }

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let connections: Connections = Arc::new(Mutex::new(HashMap::new()));
    rocket::build()
        .manage(connections)
        .mount("/", routes![
            echo_channel, 
            signin, 
            auth, 
            quit, 
            get_messages
        ])
}

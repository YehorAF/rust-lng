import json
import streamlit as st
import time
from streamlit.runtime.scriptrunner import add_script_run_ctx,get_script_run_ctx
import httpx
import os
import dotenv
import websocket
import threading


def on_message(ws: websocket.WebSocket, msg):
    msgs = st.session_state.get("messages", default=[])
    msgs.append(json.loads(msg))
    st.session_state.update({"messages": msgs})


def on_error(ws, error):
    st.error(f"Error: {error}")


def on_close(ws, close_status_code, close_msg):
    st.info("WebSocket connection closed.")


def on_open(ws):
    st.success("Connection!")
    st.session_state.update({"ws": ws})


def run_websocket(url):
    ws = websocket.WebSocketApp(
        url,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close,
        on_open=on_open,
    )
    ws.run_forever()
    st.session_state.update({"ws": ws})


def auth_widget():
    name = st.text_input("Ім'я", key="sign_key")
    password = st.text_input("Пароль", key="password", type="password")
    submit_btn = st.button("Авторизуватись")

    if submit_btn:
        origin = os.environ["ORIGIN"]
        response = httpx.post(
            f"{origin}/auth",
            data={"name": name, "password": password}
        )
        data = response.json()

        if data.get("error"):
            st.error(data.get("detail"))
        else:
            session = response.cookies.get("session")
            st.session_state.update({
                "session": session,
                "name": name,
                "user_id": data["users"][0]["_id"]["$oid"],
                "current_state": "chat"
            })
            st.rerun()


def sign_widget():
    name = st.text_input("Ім'я", key="auth_name")
    first_password = st.text_input("Пароль", key="first_password", type="password")
    second_password = st.text_input("Повторіть пароль", key="second_password", type="password")
    submit_btn = st.button("Зареєструватись")

    if submit_btn:
        if first_password != second_password:
            st.error("Паролі мають відповідати")
        elif len(name) < 4:
            st.error("Поле ім'я має містити більше 3 символів")
        elif len(first_password) < 8:
            st.error("Поле пароль має містити більше 7 символів")
        else:
            origin = os.environ["ORIGIN"]
            response = httpx.post(
                f"{origin}/signin",
                data={"name": name, "password": first_password}
            )
            data = response.json()

            if data.get("error"):
                st.error(data.get("detail"))
            else:
                session = response.cookies.get("session")
                st.session_state.update({
                    "session": session,
                    "name": name,
                    "user_id": data["users"][0]["_id"]["$oid"],
                    "current_state": "chat"
                })
                st.rerun()


def get_msgs():
    origin = os.environ["ORIGIN"]
    response = httpx.get(
        f"{origin}/messages?take=100&skip=0",
        cookies={"session": st.session_state["session"]}
    )
    res = response.json()

    return res["messages"]


def chat():
    origin = os.environ["WS_ORIGIN"]
    if "ws" not in st.session_state or not st.session_state.ws:
        user_id = st.session_state.user_id
        add_script_run_ctx(
            threading.Thread(
                target=run_websocket,
                args=(f"{origin}/ws/{user_id}",),
                daemon=True
            )
        ).start()

    if not st.session_state.get("messages"):
        msgs = get_msgs()
        st.session_state.update({"messages": msgs})

    message = st.sidebar.text_input("Напишіть повідомлення")
    send_btn = st.sidebar.button("Надіслати", key="send_btn")
    if send_btn:
        if "ws" in st.session_state and st.session_state.ws:
            user_id = st.session_state.get("user_id")
            name = st.session_state.get("name")
            st.session_state.ws.send(json.dumps({
                "content": message,
                "user": {
                    "id": user_id,
                    "name": name
                }
            }))
            st.rerun()
        else:
            st.warning("З'єднання з WebSocket не встановлено!")

    prep_msg_amount = 0
    while True:
        msgs = st.session_state.get("messages")
        if prep_msg_amount != len(msgs):
            for msg in msgs[prep_msg_amount:]:
                st.divider()
                st.caption(msg["user"]["name"])
                st.write(msg["content"])
            prep_msg_amount = len(msgs)
        time.sleep(0.5)


def pages():
    if st.session_state.current_state == "auth":
        auth_widget()
    elif st.session_state.current_state == "signin":
        sign_widget()
    elif st.session_state.current_state == "chat":
        chat()


# def message_handler():
#     for msg in st.session_state.get("messages") or []:
#         st.write(msg)


def main():
    if not os.getenv("ORIGIN"):
        dotenv.load_dotenv()

    if "current_state" not in st.session_state:
        st.session_state.current_state = "auth"

    c1, c2 = st.columns(2)
    
    auth_btn = c1.button("Авторизуватись", key="auth_btn")
    sign_btn = c2.button("Зареєструватись", key="sign_btn")
    # chat_btn = c3.button("Перейти до чату", key="chat_btn")

    if auth_btn:
        st.session_state.update({"current_state": "auth"})

    if sign_btn:
        st.session_state.update({"current_state": "signin"})

    pages()


if __name__ == "__main__":
    main()
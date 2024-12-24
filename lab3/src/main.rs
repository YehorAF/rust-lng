use std::rc::Rc;
use std::cell::RefCell;
use gtk::{
    prelude::*,  FileChooserAction, FileChooserDialog, ApplicationWindow, Box, Button, CheckButton, Entry, ListBox, Orientation, PolicyType, ScrolledWindow,
};
use gtk::{glib, Application};

mod components;

use crate::components::crud::LocalStorage;

const APP_ID: &str = "org.gtk_rs.lab3";

fn add_new_task(
    task_id: u64,
    task_name_val: String,
    is_completed: bool,
    storage: Rc<RefCell<LocalStorage>>,
    task_box: Rc<RefCell<ListBox>>,
    // state: Rc<RefCell<String>>
) 
// -> (gtk::Entry, gtk::Button, gtk::Button, Rc<gtk::CheckButton>) 
-> Box {
    let task_name = Entry::builder().text(task_name_val).build();
    let update_btn = Button::builder().label("update").build();
    let delete_btn = Button::builder().label("delete").build();
    let complete_btn = Rc::new(
        CheckButton::builder().label("complete").active(is_completed).build()
    );

    let complete_btn_clone = Rc::clone(&complete_btn);
    let storage_clone = Rc::clone(&storage);
    let task_name_clone = task_name.clone();
    let task_box_clone = Rc::clone(&task_box);
    // let state_clone = Rc::clone(&state);
    update_btn.connect_clicked(move |_| {
        let task_name_val = task_name_clone.text().to_string();
        let is_completed = complete_btn_clone.is_active();

        {
            let mut storage_current = storage_clone.borrow_mut();
            storage_current.update_task(task_id, Some(task_name_val), is_completed);
        }

        show_all_tasks(
            Rc::clone(&storage_clone), 
            Rc::clone(&task_box_clone), 
            None
            // storage_current.state.clone()
        );
    });

    let storage_clone = Rc::clone(&storage);
    let task_box_clone = Rc::clone(&task_box);
    // let state_clone = Rc::clone(&state);
    delete_btn.connect_clicked(move |_| {
        {
            let mut storage_current = storage_clone.borrow_mut();
            storage_current.delete_task(task_id);
        }

        show_all_tasks(
            Rc::clone(&storage_clone), 
            Rc::clone(&task_box_clone), 
            None
            // storage_current.state.clone()
            // Rc::clone(&state_clone)
        );
    });

    let storage_clone = Rc::clone(&storage);
    let complete_btn_clone = Rc::clone(&complete_btn);
    let task_box_clone = Rc::clone(&task_box);
    // let state_clone = Rc::clone(&state);
    complete_btn.connect_toggled(move |_| {
        {
            let mut storage = storage_clone.borrow_mut();
            let is_completed = complete_btn_clone.is_active();
            storage.update_task(task_id,None, is_completed);
        }

        show_all_tasks(
            Rc::clone(&storage_clone), 
            Rc::clone(&task_box_clone),  
            None
            // storage.state.clone()
            // Rc::clone(&state_clone)
        );
    });

    let hbox= Box::new(Orientation::Horizontal, 3);
    hbox.append(&update_btn);
    hbox.append(&delete_btn);
    hbox.append(&*complete_btn);

    let vbox = Box::new(Orientation::Vertical, 2);
    vbox.append(&task_name);
    vbox.append(&hbox);

    vbox
    // (task_name, update_btn, delete_btn, complete_btn)
}


fn show_all_tasks(
    storage_clone: Rc<RefCell<LocalStorage>>,
    task_box_clone: Rc<RefCell<ListBox>>,
    state: Option<String>
    // state_clone: Rc<RefCell<String>>
) {
    let task_list = {
        let mut storage = storage_clone.borrow_mut();
        let mut is_current = false;
        let mut is_completed = false;
        
        if state.is_some() {
            match state.as_deref() {
                Some("current") => is_current = true,
                Some("completed") => is_completed = true,
                _ => (),
            };
        } else {
            match storage.get_state().as_str() {
                "current" => is_current = true,
                "completed" => is_completed = true,
                _ => (),
            };
        }

        if is_current {
            storage.set_current();
        } else if is_completed {
            storage.set_completed();
        }
        storage.select_task_list(is_current, is_completed)
    };

    {
        let task_box = task_box_clone.borrow_mut();
        while let Some(child) = task_box.first_child() {
            task_box.remove(&child);
        }
    }

    let task_box = task_box_clone.borrow_mut();
    for task in task_list {
        let el = add_new_task(
            task.id,
            task.name,
            task.is_completed,
            Rc::clone(&storage_clone),
            Rc::clone(&task_box_clone),
        );
        task_box.append(&el);
    }
}

fn set_file_dialog(
    window: Rc<ApplicationWindow>,
    storage: Rc<RefCell<LocalStorage>>,
    task_box: Rc<RefCell<ListBox>>,
    action: FileChooserAction
) {
    let nvec = vec![
        ("Cancel", gtk::ResponseType::Cancel),
        ("Open", gtk::ResponseType::Accept)
    ];

    let dialog = FileChooserDialog::new(
        Some("Open File"),
        Some(&*window),
        action,
        &nvec
    );

    dialog.set_modal(true);

    dialog.show();

    let storage_clone = Rc::clone(&storage);
    dialog.connect_response(move |dialog, response| {
        {
            let mut storage = storage_clone.borrow_mut();
            if response == gtk::ResponseType::Accept {
                if let Some(path) = dialog.file() {
                    if action == FileChooserAction::Open {
                        storage.file_to_task(path.path().unwrap().to_str().unwrap());
                    } else if action == FileChooserAction::Save {
                        storage.task_to_file(path.path().unwrap().to_str().unwrap());
                    }
                }
            }
        }
        dialog.destroy(); 
        show_all_tasks(
            Rc::clone(&storage),
            Rc::clone(&task_box),
            None
        );
    });
}

fn build_ui(app: &Application) {
    
    let window = Rc::new(
        ApplicationWindow::builder()
        .application(app)
        .title("Tasks")
        // .child(&vbox)
        .build()
    );
    let storage =  Rc::new(RefCell::new(LocalStorage {
        current_id: 0,
        tasks: Vec::new(),
        state: String::from("current"),
    }));
    let task_box = Rc::new(RefCell::new(ListBox::new()));
    // let state = Rc::new(RefCell::new(String::from("current")));

    let current_btn = Button::builder().label("Current").build();
    let completed_btn = Button::builder().label("Completed").build();

    let  filter_tab = Box::new(Orientation::Horizontal, 2);
    filter_tab.append(&current_btn);
    filter_tab.append(&completed_btn);

    let task_box_clone = Rc::clone(&task_box);
    let task_list = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .min_content_width(600)
        .min_content_height(600)
        .child(&*task_box_clone.borrow_mut())
        .build();

    let task_name_field = Entry::builder().text("").build();
    let create_btn = Rc::new(Button::builder().label("+").build());
    let import_btn = Button::builder().label("^").build();
    let export_btn = Button::builder().label(".").build();

    let hbox = Box::new(Orientation::Horizontal, 2);
    hbox.append(&task_name_field);
    hbox.append(&*create_btn);
    hbox.append(&import_btn);
    hbox.append(&export_btn);

    let storage_clone = Rc::clone(&storage);
    let task_box_clone = Rc::clone(&task_box);
    create_btn.connect_clicked(move |_| {
        let task_name_val = task_name_field.text().to_string().clone();
        let mut storage = storage_clone.borrow_mut();
        let task_box = task_box_clone.borrow_mut();
        let task_id = storage.create_task(task_name_val.clone());
        let vbox = add_new_task(
            task_id, 
            task_name_val, 
            false,
            Rc::clone(&storage_clone),
            Rc::clone(&task_box_clone),
        );

        task_box.append(&vbox);
    });

    let window_clone = Rc::clone(&window);
    let storage_clone = Rc::clone(&storage);
    let task_box_clone = Rc::clone(&task_box);
    import_btn.connect_clicked(move |_|{
        set_file_dialog(
            Rc::clone(&window_clone), 
            Rc::clone(&storage_clone),
            Rc::clone(&task_box_clone),
            FileChooserAction::Open
        );
    });

    let window_clone = Rc::clone(&window);
    let storage_clone = Rc::clone(&storage);
    let task_box_clone = Rc::clone(&task_box);
    export_btn.connect_clicked(move |_|{
        set_file_dialog(
            Rc::clone(&window_clone), 
            Rc::clone(&storage_clone),
            Rc::clone(&task_box_clone),
            FileChooserAction::Save
        );
    });

    let storage_clone = Rc::clone(&storage);
    let task_box_clone = Rc::clone(&task_box);
    let create_btn_clone = Rc::clone(&create_btn);
    current_btn.connect_clicked(move |_| {
        show_all_tasks(
            Rc::clone(&storage_clone), 
            Rc::clone(&task_box_clone),
            Some(String::from("current")) 
        );
        create_btn_clone.set_visible(true);
    });

    let storage_clone = Rc::clone(&storage);
    let task_box_clone = Rc::clone(&task_box);
    let create_btn_clone = Rc::clone(&create_btn);
    completed_btn.connect_clicked(move |_| {
        show_all_tasks(
            Rc::clone(&storage_clone), 
            Rc::clone(&task_box_clone), 
            Some(String::from("completed"))
        );
        create_btn_clone.set_visible(false);
    });

    let vbox = Box::new(Orientation::Vertical, 3);
    vbox.append(&filter_tab);
    vbox.append(&task_list);
    vbox.append(&hbox);

    window.set_child(Some(&vbox));
    window.present();
}

fn main() {
    let app = Application::builder()
    .application_id(APP_ID)
    .build();

    app.connect_activate(build_ui);
    app.run();
}
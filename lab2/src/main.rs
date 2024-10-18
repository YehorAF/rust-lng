use gtk::{
    prelude::*, ApplicationWindow, Box, Button, Entry, ListBox, Orientation, ScrolledWindow, Text, PolicyType
};
use gtk::{glib, Application};

const APP_ID: &str = "org.gtk_rs.lab2";

static COPS: [char; 5] = ['+', '-', '*', '/', '^'];
static SOPS: [&str; 5] = ["+", "-", "*", "/", "^"];
static UN: [&str; 2] = ["+", "-"];
static EPS: f64 = 0.0000000001;

fn calculate_expr(
    oprnd1: f64, oprnd2: f64, op: String
) -> Result<f64, &'static str> {
    return match op.as_str() {
        "+" => Ok(oprnd1 + oprnd2),
        "-" => Ok(oprnd1 - oprnd2),
        "*" => Ok(oprnd1 * oprnd2),
        "/" => if oprnd2.abs() > EPS {Ok(oprnd1 / oprnd2)}
               else {Err("divisaion by 0")},
        "^" => Ok(oprnd1.powf(oprnd2)),
        _ => Err("invalid operator"),
    };
}

fn get_operator_priority(op: String) -> i8 {
    if op == "+" || op == "-" {
        return  1;
    } else if op == "*" || op == "/" {
        return  2;
    } else if op == "^" {
        return 3;
    }
    
    return 0;
}

fn parse_expression_to_tokens(expr: &str) -> Result<Vec<String>, &'static str> {
    let bexpr: Vec<char> = expr.chars().collect();
    let mut s = String::from("");
    let mut el_type = "empty";
    let mut tokens= Vec::new();

    for ch in bexpr {
        if el_type != "float" && ch.is_digit(10) {
            s.push(ch);
            el_type = "int";
        } else if COPS.contains(&ch) {
            if !s.is_empty() {
                tokens.push(s.clone());
            }
            tokens.push(ch.to_string());
            s.clear();
            el_type = "op";
        } else if el_type == "int" && ch == '.' {
            s.push(ch);
            el_type = "float"
        } else if el_type == "float" && ch.is_digit(10){
            s.push(ch);
        } else {
            return Err("invalid token");
        }
    }
    tokens.push(s.clone());

    return Ok(tokens);
}

fn parse_tokens_to_rpn(tokens: Vec<String>) -> Result<Vec<String>, &'static str> {
    let mut expr_ops: Vec<String> = Vec::new();
    let mut rpn_expr: Vec<String> = Vec::new();

    for token in tokens {
        if token.parse::<f64>().is_ok() {
            rpn_expr.push(token.clone());
        }
        else if !SOPS.contains(&token.as_str()) {
            return Err("invalid token");
        } else if expr_ops.is_empty(){
            expr_ops.push(token.clone());
        } else if 
            get_operator_priority(expr_ops.last().unwrap().clone()) < 
            get_operator_priority(token.clone()) 
        {
            expr_ops.push(token.clone());
        } else {
            while 
                !expr_ops.is_empty() && 
                get_operator_priority(expr_ops.last().unwrap().clone()) >= 
                get_operator_priority(token.clone()) 
            {
                rpn_expr.push(expr_ops.pop().unwrap().clone());
            }
            expr_ops.push(token.clone());
        }
    }

    for op in expr_ops.iter().rev() {
        rpn_expr.push(op.clone());
    }

    return Ok(rpn_expr);

}

fn calculate_rpn(rpn: Vec<String>) -> Result<f64, &'static str> {
    let mut nums: Vec<f64> = Vec::new();

    for token in rpn {
        let res = token.parse::<f64>();
        let is_un = UN.contains(&token.as_str());
        let is_op = SOPS.contains(&token.as_str());
        if res.is_ok() {
            nums.push(res.unwrap());
        } else if is_un && nums.len() < 2 {
            let oprnd = nums.pop().unwrap();
            
            match token.as_str() {
                "-" => nums.push(-oprnd),
                _ => {}
            };
        } else if is_op {
            let oprnd2 = nums.pop().unwrap();
            let oprnd1 = nums.pop().unwrap();

            match calculate_expr(oprnd1, oprnd2, token) {
                Ok(res) => nums.push(res),
                Err(err) => return Err(err),
            };
        } else {
            return Err("invalid token");
        }
    }

    return Ok(nums[0]);
}

fn calc(expr: &str) -> Result<f64, String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut rpn: Vec<String> = Vec::new();

    match parse_expression_to_tokens(expr) {
        Ok(res) => tokens = res,
        Err(err) => return Err(format!("Parse Error: {err}")),
    }
    println!("Tokens: {}", tokens.join(" "));

    match parse_tokens_to_rpn(tokens){
        Ok(res) => rpn = res,
        Err(err) => return Err(format!("RPN Error: {err}")),
    }

    println!("RPN: {}", rpn.join(" "));

    match calculate_rpn(rpn) {
        Ok(res) => return Ok(res),
        Err(err) => return Err(format!("Calculate Error: {err}")),
    }
}

fn build_ui(app: &Application) {
    let res_box = ListBox::new();
    let err_text = Text::builder().text("").build();
    let field_input = Entry::builder().build();
    let btn = Button::builder().label("=").build();

    let cloned_res_box = res_box.clone();
    let cloned_err_text = err_text.clone();
    let cloned_field_input = field_input.clone();

    btn.connect_clicked(move |_| {
        let expr = cloned_field_input.text();
        let res = calc(expr.as_str());
        // let res = evalexpr::eval(&expr);

        match res {
            Ok(res_) => cloned_res_box.append(
                &Text::builder().text(
                    &format!("{expr} = {res_}")
                ).build()
            ),
            Err(err) => cloned_err_text
                .set_text(format!("{err}").as_str()),
        }
    });

    let res_list = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .min_content_width(200)
        .child(&res_box)
        .build();

    let vbox = Box::new(Orientation::Vertical, 5);
    vbox.append(&res_list);
    vbox.append(&field_input);
    vbox.append(&btn);
    vbox.append(&err_text);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("RPNCalc")
        .child(&vbox)
        .build();

    window.present();
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}
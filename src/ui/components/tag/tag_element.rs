use dioxus::prelude::*;

#[derive(PartialEq, Eq, Props)]
pub struct Props {
    #[props(required)]
    pub text: String,
    #[props(required)]
    pub color: String,
}

pub fn TagElement() -> Element {  

}
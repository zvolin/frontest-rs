use gloo::timers::future::sleep;
use std::time::Duration;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

pub fn element_to_aria_roles(elem: &HtmlElement) -> Vec<&'static str> {
    match elem.tag_name().to_lowercase().as_str() {
        "article" => vec!["article"],
        "button" => vec!["button"],
        "td" => vec!["cell", "gridcell"],
        "select" => vec!["combobox", "listbox"],
        "menuitem" => vec!["command", "menuitem"],
        "dd" => vec!["definition"],
        "figure" => vec!["figure"],
        "form" => vec!["form"],
        "table" => vec!["grid", "table"],
        "fieldset" => vec!["group"],
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => vec!["heading"],
        "img" => vec!["img"],
        "a" | "link" => vec!["link"],
        "ol" | "ul" => vec!["list"],
        "li" => vec!["listitem"],
        "nav" => vec!["navigation"],
        "option" => vec!["option"],
        "frame" => vec!["region"],
        "rel" => vec!["roletype"],
        "tr" => vec!["row"],
        "tbody" | "tfoot" | "thead" => vec!["rowgroup"],
        "hr" => vec!["separator"],
        "dt" | "dfn" => vec!["term"],
        "textarea" => vec!["textbox"],
        "input" => match elem.get_attribute("type").as_deref().unwrap_or("") {
            "checkbox" => vec!["checkbox"],
            "radio" => vec!["radio"],
            "search" => vec!["searchbox"],
            "text" => vec!["textbox"],
            _ => vec![],
        },
        "th" => match elem.get_attribute("scope").as_deref().unwrap_or("") {
            "row" => vec!["rowheader"],
            _ => vec!["columnheader"],
        },
        _ => vec![],
    }
}

async fn tick() {
    sleep(Duration::ZERO).await;
}

#[cfg(feature = "yew")]
pub mod yew {
    use super::*;
    use ::yew::prelude::*;

    #[derive(Properties, PartialEq)]
    struct WrapperProps {
        content: Html,
    }

    #[function_component(Wrapper)]
    fn wrapper(props: &WrapperProps) -> Html {
        props.content.clone()
    }

    pub async fn render(content: Html) -> ElementHandle {
        ::yew::start_app_with_props_in_element::<Wrapper>(
            gloo::utils::document().get_element_by_id("output").unwrap(),
            WrapperProps { content },
        );

        tick().await;
        ElementHandle {
            element: gloo::utils::document().get_element_by_id("output").unwrap(),
        }
    }
}

pub struct ElementHandle {
    pub element: Element,
}

impl ElementHandle {
    pub fn get_by_text(&self, text: &str) -> Option<HtmlElement> {
        let selected = self.element.query_selector_all("*").unwrap();
        // Get all nodes matching given text
        let matching = (0..selected.length())
            .filter_map(|idx| selected.get(idx))
            .filter_map(|node| node.dyn_into::<HtmlElement>().ok())
            .filter(|element| element.inner_text().contains(text))
            .collect::<Vec<_>>();
        // Remove a node if it was included because of matching child
        let mut matching = matching
            .iter()
            .cloned()
            .filter(|elem| {
                matching
                    .iter()
                    .any(|other| elem != other && !elem.contains(Some(other)))
            })
            .collect::<Vec<_>>();

        match matching.len() {
            0 => None,
            1 => Some(matching.pop().unwrap()),
            _ => panic!("Found more than one element matching text '{}'", text),
        }
    }

    pub fn get_by_role(&self, role: &str) -> Option<HtmlElement> {
        let selected = self.element.query_selector_all("*").unwrap();
        // Get all nodes matching given text
        let mut matching = (0..selected.length())
            .filter_map(|idx| selected.get(idx))
            .filter_map(|node| node.dyn_into::<HtmlElement>().ok())
            .filter(|element| {
                if element_to_aria_roles(&element).contains(&role) {
                    true
                } else if let Some(role_attr) = element.get_attribute("role") {
                    role_attr == role
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();

        match matching.len() {
            0 => None,
            1 => Some(matching.pop().unwrap()),
            _ => panic!("Found more than one element matching role '{}'", role),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::yew::prelude::*;

    #[allow(unused_imports)]
    use wasm_bindgen_test::console_log;
    use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};
    wasm_bindgen_test_configure!(run_in_browser);

    #[function_component(App)]
    fn app() -> Html {
        let counter = use_state(|| 0);
        let onclick = {
            let counter = counter.clone();
            Callback::from(move |_| counter.set(*counter + 1))
        };
        html! {
            <div>
                <input placeholder="Placeholder" />
                <p>{ format!("Value: {}", *counter) }</p>
                <button {onclick}>{ "Add" }</button>
            </div>
        }
    }

    #[test]
    async fn unit_tests() {
        let mount = yew::render(html! { <App /> }).await;
        let value = mount.get_by_text("Value").unwrap();
        let button = mount.get_by_text("Add").unwrap();

        assert_eq!("Value: 0", value.inner_text());

        button.click();
        tick().await;

        assert_eq!("Value: 1", value.inner_text());
    }
}

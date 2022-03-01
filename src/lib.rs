//! Quick and lightweight library to query and assert DOM.
//!
//! Frontest is heavily inspired by [`dom-testing-library`] and [`react-testing-library`].
//! It provides a set of queries that you can use to quickly find your elements in document
//! with respect to accessibility priorities.
//!
//! # Basic usage:
//!
//! Let's write a test for a simple [`yew`] component that displays it's value and increments it on button click.
//! ```no_run
//! # use yew::prelude::*;
//! #[function_component(Incrementable)]
//! fn incrementable() -> Html {
//!     let counter = use_state(|| 0);
//!     let onclick = {
//!         let counter = counter.clone();
//!         Callback::from(move |_| counter.set(*counter + 1))
//!     };
//!     html! {
//!         <div>
//!             <p>{ format!("Value: {}", *counter) }</p>
//!             <button {onclick}>{ "Add" }</button>
//!         </div>
//!     }
//! }
//!
//! # use wasm_bindgen_test::wasm_bindgen_test;
//! use frontest::{tick, Selector, HasText, HasRole};
//! use frontest::yew::render;
//! use yew::html;
//!
//! #[wasm_bindgen_test]
//! async fn clicking_on_button_should_increment_value() {
//!     let mount = render(html! { <Incrementable /> }).await;
//!     let value = mount.get(&HasText("Value:")).unwrap();
//!     let button = mount.get(&HasRole("button")).unwrap();
//!
//!     assert_eq!("Value: 0", value.inner_text());
//!
//!     button.click();
//!
//!     assert_eq!("Value: 1", value.inner_text());
//! }
//! ```
use gloo::timers::future::sleep;
use std::time::Duration;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

/// Returns the list of aria roles for a given [`HtmlElement`]
///
/// Aria role is a semantic meaning of an element.
/// It provides a web site with an [`accessibility`].
/// List of assigned roles was shamelessly taken from [aria-query](https://www.npmjs.com/package/aria-query).
///
/// | Tag                             | Roles             |
/// |---------------------------------|-------------------|
/// | `<article>`                     | article           |
/// | `<button>`                      | button            |
/// | `<td>`                          | cell, gridcell    |
/// | `<select>`                      | combobox, listbox |
/// | `<menuitem>`                    | command, menuitem |
/// | `<dd>`                          | definition        |
/// | `<figure>`                      | figure            |
/// | `<form>`                        | form              |
/// | `<table>`                       | grid, table       |
/// | `<fieldset>`                    | group             |
/// | `<h1> <h2> <h3> <h4> <h5> <h6>` | heading           |
/// | `<img>`                         | img               |
/// | `<a> <link>`                    | link              |
/// | `<ol> <ul>`                     | list              |
/// | `<li>`                          | listitem          |
/// | `<nav>`                         | navigation        |
/// | `<option>`                      | option            |
/// | `<frame>`                       | region            |
/// | `<rel>`                         | roletype          |
/// | `<tr>`                          | row               |
/// | `<tbody> <tfoot> <thead>`       | rowgroup          |
/// | `<hr>`                          | separator         |
/// | `<dt> <dfn>`                    | term              |
/// | `<textarea>`                    | textbox           |
/// | `<input type=button>`           | button            |
/// | `<input type=checkbox>`         | checkbox          |
/// | `<input type=radio>`            | radio             |
/// | `<input type=search>`           | searchbox         |
/// | `<input type=text>`             | textbox           |
/// | `<th scope=row>`                | rowheader         |
/// | `<th>`                          | columnheader      |
///
/// # Example:
///
/// ```no_run
/// # use frontest::element_to_aria_roles;
/// # use wasm_bindgen::JsCast;
/// # use web_sys::HtmlElement;
/// # let document = gloo::utils::document();
/// // <a tag="foo" href="...">"Click me"</a>
/// let anchor: HtmlElement = document.get_element_by_id("foo")
///     .unwrap()
///     .unchecked_into();
/// assert_eq!(element_to_aria_roles(&anchor), vec!["link"]);
/// ```
/// [`accessibility`]: https://developer.mozilla.org/en-US/docs/Web/Accessibility
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
            "button" => vec!["button"],
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

/// Trait implemented by types that can be used as a predicate for [`HtmlElement`].
///
/// One can implement this trait to create a custom [`Matcher`]s.
///
/// # Example:
/// ```no_run
/// # use web_sys::HtmlElement;
/// # use frontest::{HasRole, Joinable, Selector};
/// # use gloo::utils::{body, document};
/// use frontest::Matcher;
///
/// struct IsHidden;
///
/// impl Matcher for IsHidden {
///     fn matches(&self, elem: &HtmlElement) -> bool {
///         elem.hidden()
///     }
/// }
///
/// let div = document().create_element("div").unwrap();
/// div.set_inner_html(
///     r#"<button hidden>
///         Yayyy frontend in rust!
///     </button>"#
/// );
/// body().append_child(&div).unwrap();
///
/// let hidden_button = div.get(&IsHidden.and(HasRole("button"))).unwrap();
///
/// assert!(hidden_button.inner_html().contains("in rust"));
/// ```
pub trait Matcher {
    /// Returns `true` if the element was matched by [`Matcher`].
    fn matches(&self, elem: &HtmlElement) -> bool;
}

#[cfg(test)]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn doctest_matcher() {
    use crate::Matcher;
    use crate::{HasRole, Joinable, Selector};
    use gloo::utils::{body, document};
    use web_sys::HtmlElement;

    struct IsHidden;

    impl Matcher for IsHidden {
        fn matches(&self, elem: &HtmlElement) -> bool {
            elem.hidden()
        }
    }

    let div = document().create_element("div").unwrap();
    div.set_inner_html(
        r#"<button hidden>
            Yayyy frontend in rust!
        </button>"#,
    );
    body().append_child(&div).unwrap();

    let hidden_button = div.get(&IsHidden.and(HasRole("button"))).unwrap();

    assert!(hidden_button.inner_html().contains("in rust"));
}

/// Matches components that have visible text that contains given substring.
///
/// [`HasText`] uses [`inner_text`] under the hood and is case-sensitive.
/// It will match elements by their content as presented for user.
/// All css rules applies eg. those switching text content, case or visibility.
/// Remember that for this experience you need to insert an element somewhere into DOM.
///
/// # Example:
///
/// ```no_run
/// # use gloo::utils::{body, document};
/// # use frontest::{HasText, Selector};
/// let div = document().create_element("div").unwrap();
/// div.set_inner_html(
///     r#"<div>
///         <button>I am</button>
///         <button style="visibility: hidden;">Blue</button>
///     </div>"#,
/// );
/// // Without this line, the last assert will panic as css rules won't be applied.
/// body().append_child(&div).unwrap();
///
/// assert!(div.get(&HasText("I am")).is_some());
/// assert!(div.get(&HasText("i am")).is_none());
/// assert!(div.get(&HasText("Blue")).is_none());
/// ```
/// [`inner_text`]: web_sys::HtmlElement::inner_text
pub struct HasText<'a>(pub &'a str);

impl<'a> Matcher for HasText<'a> {
    fn matches(&self, elem: &HtmlElement) -> bool {
        elem.inner_text().contains(self.0) && {
            let children_len = elem.children().length();
            !(0..children_len)
                .filter_map(|n| elem.children().item(n))
                .filter_map(|child| child.dyn_into::<HtmlElement>().ok())
                .any(|child| child.inner_text().contains(self.0))
        }
    }
}

#[cfg(test)]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn doctest_has_text() {
    use crate::{HasText, Selector};
    use gloo::utils::{body, document};
    let div = document().create_element("div").unwrap();
    div.set_inner_html(
        r#"<div>
            <button>I am</button>
            <button style="visibility: hidden;">Blue</button>
        </div>"#,
    );
    // Without this line, the last assert will panic as css rules won't be applied.
    body().append_child(&div).unwrap();

    assert!(div.get(&HasText("I am")).is_some());
    assert!(div.get(&HasText("i am")).is_none());
    assert!(div.get(&HasText("Blue")).is_none());
}

/// Matches components that have given aria role.
///
/// This is by far the best method for finding components as it searches for elements in the [`accessibility tree`].
/// You should always prefere something like `.get(&HasRole("button").and(HasText("Add")))` over the alternavies.
/// Aria-roles currently support semantic tag to role deduction with [`element_to_aria_roles`].
/// It currently doesn't support any of [`aria_attribute_types`] or implicit role deduction.
/// Support for those is planned as much as it can be at this age of project.
///
/// # Example:
///
/// ```no_run
/// # use gloo::utils::{body, document};
/// # use frontest::{HasRole, Selector};
/// let div = document().create_element("div").unwrap();
/// div.set_inner_html(
///     r#"<div>
///         <button>Rust</button>
///         <input type="button">Is</input>
///         <div role="button">Fun</input>
///     </div>"#,
/// );
/// body().append_child(&div).unwrap();
///
/// assert_eq!(body().get_all(&HasRole("button")).len(), 3);
/// ```
/// [`accessibility_tree`]: https://developer.mozilla.org/en-US/docs/Glossary/Accessibility_tree
/// [`aria_attribute_types`]: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes#aria_attribute_types
pub struct HasRole<'a>(pub &'a str);

impl<'a> Matcher for HasRole<'a> {
    fn matches(&self, elem: &HtmlElement) -> bool {
        if element_to_aria_roles(elem).contains(&self.0) {
            true
        } else if let Some(role) = elem.get_attribute("role") {
            role == self.0
        } else {
            false
        }
    }
}

#[cfg(test)]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn doctest_has_role() {
    use crate::{HasRole, Selector};
    use gloo::utils::{body, document};
    let div = document().create_element("div").unwrap();
    div.set_inner_html(
        r#"<div>
            <button>Rust</button>
            <input type="button">Is</input>
            <div role="button">Fun</input>
        </div>"#,
    );
    body().append_child(&div).unwrap();

    assert_eq!(body().get_all(&HasRole("button")).len(), 3);
}

pub trait Joinable {
    fn and<'a, 'b, F>(self, other: F) -> AndFilter<'b>
    where
        'a: 'b,
        Self: Sized + Matcher + 'a,
        F: Matcher + 'a,
    {
        AndFilter {
            filters: vec![Box::new(self), Box::new(other)],
        }
    }

    fn or<'a, 'b, F>(self, other: F) -> OrFilter<'b>
    where
        'a: 'b,
        Self: Sized + Matcher + 'a,
        F: Matcher + 'a,
    {
        OrFilter {
            filters: vec![Box::new(self), Box::new(other)],
        }
    }
}

impl<F> Joinable for F where F: Matcher {}

pub struct AndFilter<'a> {
    filters: Vec<Box<dyn Matcher + 'a>>,
}

impl<'a> Matcher for AndFilter<'a> {
    fn matches(&self, elem: &HtmlElement) -> bool {
        self.filters.iter().all(|f| f.matches(elem))
    }
}

pub struct OrFilter<'a> {
    filters: Vec<Box<dyn Matcher + 'a>>,
}

impl<'a> Matcher for OrFilter<'a> {
    fn matches(&self, elem: &HtmlElement) -> bool {
        self.filters.iter().any(|f| f.matches(elem))
    }
}

pub trait Selector {
    fn get<M: Matcher>(&self, rules: &M) -> Option<HtmlElement>;
    fn get_all<M: Matcher>(&self, rules: &M) -> Vec<HtmlElement>;
}

impl Selector for Element {
    fn get<M: Matcher>(&self, matcher: &M) -> Option<HtmlElement> {
        let selected = self.query_selector_all("*").unwrap();
        // Get all nodes matching given text
        let mut preprocessed = (0..selected.length())
            .filter_map(|idx| selected.get(idx))
            .filter_map(|node| node.dyn_into::<HtmlElement>().ok())
            .filter(|e| matcher.matches(e))
            .collect::<Vec<_>>();

        match preprocessed.len() {
            0 => None,
            1 => Some(preprocessed.pop().unwrap()),
            _ => panic!("Found more than one element."),
        }
    }

    fn get_all<M: Matcher>(&self, matcher: &M) -> Vec<HtmlElement> {
        let selected = self.query_selector_all("*").unwrap();
        // Get all nodes matching given text
        (0..selected.length())
            .filter_map(|idx| selected.get(idx))
            .filter_map(|node| node.dyn_into::<HtmlElement>().ok())
            .filter(|e| matcher.matches(e))
            .collect::<Vec<_>>()
    }
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

    pub async fn render(content: Html) -> Element {
        ::yew::start_app_with_props_in_element::<Wrapper>(
            gloo::utils::document().get_element_by_id("output").unwrap(),
            WrapperProps { content },
        );

        gloo::utils::document().get_element_by_id("output").unwrap()
    }
}

/// Preempt execution of current task to let the js's main thread do things like re-render.
///
/// # Warning:
/// I'm currently unsure in which condition tick is required. Most cases should work without the need of using it.
pub async fn tick() {
    sleep(Duration::ZERO).await;
}

#[cfg(test)]
mod tests {
    use super::yew::render;
    use super::*;
    use ::yew::prelude::*;
    use futures::future::FutureExt;

    #[allow(unused_imports)]
    use wasm_bindgen_test::console_log;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
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
                <p>{ format!("Value: {}", *counter) }</p>
                <button {onclick}>{ "Add" }</button>
            </div>
        }
    }

    #[wasm_bindgen_test]
    async fn unit_tests() {
        render(html! { <App /> })
            .then(|mount| async move {
                let button = mount.get(&HasRole("button").and(HasText("Add"))).unwrap();
                let value = mount.get(&HasText("Chuj").or(HasText("Value"))).unwrap();

                assert_eq!("Value: 0", value.inner_text());

                button.click();

                assert_eq!("Value: 1", value.inner_text());
            })
            .await;
    }
}

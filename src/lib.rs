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
//! # #[cfg(feature = "yew")]
//! # {
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
//! use frontest::{Query, HasText, HasRole};
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
//!     button.click();
//!     assert_eq!("Value: 1", value.inner_text());
//! }
//! # }
//! ```
//!
//! # About testing:
//!
//! This library aims to allow developers to test their application in a way that a user would interact with it.
//! For this purpose it is recommended to prioritize certain queries above another.
//! Currently only two matchers are implemented. More will be available in future releases.
//! Matchers should be prioritized like so:
//! - [`HasRole`] Should always be used where possible. It allows accessing elements that are exposed into accessibility tree.
//! - [`HasText`] Can be used to select non-interactive components or further restrict other queries.
//!
//! # Matchers:
//!
//! Matchers are predicates for [`HtmlElement`]. They return [`true`] if given element suffices some criteria
//! or [`false`] otherwise.
//!
//! Using the matcher [`Not`] and methods from [`Joinable`] trait it is possible to combine multiple matchers into
//! a logical expression.
//!
//! # Integration:
//! Tests should be run using [`wasm-bindgen-test`]. It allows running them directly in browsers or in node-js.
//!
//! Currently this crate provides a [`render`] function that allows for quickly rendering any [`html`] created with [`yew`].
//! It was choosen to render the html instead of directly taking a component so it is easier to wrap them with [`ContextProvider`] and so on.
//!
//! ## Example:
//! ```no_run
//! # #[cfg(feature = "yew")]
//! # {
//! # use wasm_bindgen_test::wasm_bindgen_test;
//! use frontest::yew::render;
//! use yew::prelude::*;
//! #
//! # #[function_component(Baz)]
//! # fn baz() -> Html { html! {} }
//! #
//! # #[derive(Clone, PartialEq)]
//! # struct Bar {}
//!
//! #[wasm_bindgen_test]
//! async fn foo() {
//!     let elem = render(html! {
//!         <ContextProvider<Bar> context={Bar {}}>
//!             <Baz />
//!         </ContextProvider<Bar>>
//!     }).await;
//! }
//! # }
//! ```
//!
//! ## Warning:
//!
//! [`wasm-bindgen-test`] runs all tests sequentially and let them manipulate real DOM.
//! However it doesn't recreate full DOM for each test, so things done in one test may impact others.
//! Always make sure you are doing a proper cleanup of DOM after your tests eg. remove mounted child element.
//! Hopefully in future this library will provide some kind of RAII for running tests.
//!
//! [`dom-testing-library`]: https://testing-library.com/docs/dom-testing-library/intro
//! [`react-testing-library`]: https://testing-library.com/docs/react-testing-library/intro
//! [`wasm-bindgen-test`]: https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/usage.html
//! [`render`]: yew::render
//! [`html`]: ::yew::html!
//! [`ContextProvider`]: ::yew::context::ContextProvider
use gloo::timers::future::sleep;
use std::time::Duration;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

/// Returns the list of aria roles for a given [`HtmlElement`].
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
/// One can implement this trait to create custom [`Matcher`]s.
///
/// # Example:
/// ```no_run
/// # use web_sys::HtmlElement;
/// # use frontest::{HasRole, Joinable, Query};
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
    use crate::{HasRole, Joinable, Query};
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

/// Consumes a [`Matcher`] and returns a negation of it.
///
/// Utility wrapper that performs a logical `not` operation on a matcher.
///
/// # Example:
///
/// ```no_run
/// use frontest::{HasRole, HasText, Joinable, Not, Query};
/// use gloo::utils::{body, document};
/// let div = document().create_element("div").unwrap();
/// div.set_inner_html(
///     r#"<div>
///         <p>what</p>
///         <a href="/foo">is</a>
///         <button>this</button>
///     </div>"#,
/// );
/// body().append_child(&div).unwrap();
///
/// let link = div.get(&HasText("is").and(Not(HasRole("button")))).unwrap();
/// assert_eq!(&link.get_attribute("href").unwrap(), "/foo");
/// ```
pub struct Not<M: Matcher>(pub M);

impl<M: Matcher> Matcher for Not<M> {
    fn matches(&self, elem: &HtmlElement) -> bool {
        !self.0.matches(elem)
    }
}

#[cfg(test)]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn doctest_not() {
    use crate::{HasRole, HasText, Joinable, Not, Query};
    use gloo::utils::{body, document};
    let div = document().create_element("div").unwrap();
    div.set_inner_html(
        r#"<div>
            <p>what</p>
            <a href="/foo">is</a>
            <button>this</button>
        </div>"#,
    );
    body().append_child(&div).unwrap();

    let link = div.get(&HasText("is").and(Not(HasRole("button")))).unwrap();
    assert_eq!(&link.get_attribute("href").unwrap(), "/foo");
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
/// # use frontest::{HasText, Query};
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
    use crate::{HasText, Query};
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
/// You should always prefer something like `.get(&HasRole("button").and(HasText("Add")))` over the alternavies.
/// Currently only supports user assigned roles and semantic tag to role deduction with [`element_to_aria_roles`].
/// It currently doesn't support any of [`aria_attribute_types`] or implicit role deduction.
/// Support for those is planned as much as it can be at this age of project.
///
/// # Example:
///
/// ```no_run
/// # use gloo::utils::{body, document};
/// # use frontest::{HasRole, Query};
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
/// assert_eq!(div.get_all(&HasRole("button")).len(), 3);
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
    use crate::{HasRole, Query};
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

    assert_eq!(div.get_all(&HasRole("button")).len(), 3);
}

/// A trait for joining multiple matchers.
///
/// It is automatically implemented for all matchers.
/// It allows for joining matchers using `or` and `and` methods that consume both matchers
/// and returns a joined matcher. It can be chained with multiple calls.
///
/// # Example:
/// ```no_run
/// use frontest::{HasRole, HasText, Joinable, Query};
/// use gloo::utils::{body, document};
/// let div = document().create_element("div").unwrap();
/// div.set_inner_html(
///     r#"<div>
///         <button>I eat cookies</button>
///     </div>"#,
/// );
/// body().append_child(&div).unwrap();
///
/// assert!(div
///     .get(
///         &HasRole("button")
///             .and(HasText("bananas").or(HasText("apples")))
///             .or(HasText("cookies"))
///     )
///     .is_some());
/// ```
pub trait Joinable {
    /// Join two matchers by applying logical `and` operation.
    fn and<'a, 'b, M>(self, other: M) -> And<'b>
    where
        'a: 'b,
        Self: Sized + Matcher + 'a,
        M: Matcher + 'a,
    {
        And {
            filters: [Box::new(self), Box::new(other)],
        }
    }

    /// Join two matchers by applying logical `or` operation.
    fn or<'a, 'b, M>(self, other: M) -> Or<'b>
    where
        'a: 'b,
        Self: Sized + Matcher + 'a,
        M: Matcher + 'a,
    {
        Or {
            filters: [Box::new(self), Box::new(other)],
        }
    }
}

impl<M> Joinable for M where M: Matcher {}

#[cfg(test)]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn doctest_joinable() {
    use crate::{HasRole, HasText, Joinable, Query};
    use gloo::utils::{body, document};
    let div = document().create_element("div").unwrap();
    div.set_inner_html(
        r#"<div>
            <button>I eat cookies</button>
        </div>"#,
    );
    body().append_child(&div).unwrap();

    assert!(div
        .get(
            &HasRole("button")
                .and(HasText("bananas").or(HasText("apples")))
                .or(HasText("cookies"))
        )
        .is_some());
}

/// Result of joining two [`Matcher`]s by applyng a logical [`and`] operation on them.
///
/// [`and`]: Joinable::and
pub struct And<'a> {
    filters: [Box<dyn Matcher + 'a>; 2],
}

impl<'a> Matcher for And<'a> {
    fn matches(&self, elem: &HtmlElement) -> bool {
        self.filters.iter().all(|f| f.matches(elem))
    }
}

/// Result of combining two [`Matcher`]s by applyng a logical [`or`] operation on them.
///
/// [`or`]: Joinable::or
pub struct Or<'a> {
    filters: [Box<dyn Matcher + 'a>; 2],
}

impl<'a> Matcher for Or<'a> {
    fn matches(&self, elem: &HtmlElement) -> bool {
        self.filters.iter().any(|f| f.matches(elem))
    }
}

/// Allows selecting [`HtmlElement`]s using [`Matcher`]s.
///
/// By default implemented for [`Element`] where it selects it's children matching provided pattern.
pub trait Query {
    /// Tries to get a unique component. Returns [`None`] on failure and [`HtmlElement`] on success.
    ///
    /// # Panics:
    /// If more than one element is found.
    fn get<M: Matcher>(&self, rules: &M) -> Option<HtmlElement>;

    /// Returns a [`Vec`] of all components matched by a [`Matcher`].
    fn get_all<M: Matcher>(&self, rules: &M) -> Vec<HtmlElement>;
}

impl Query for Element {
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

/// A helpers when testing frontend made with [`yew`]
///
/// [`yew`]: ::yew
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

    /// Render arbitrary output of [`html`] macro, mount it into body and return mount-point [`Element`]
    ///
    /// # Example:
    /// ```no_run
    /// # use yew::prelude::*;
    /// #[function_component(Incrementable)]
    /// fn incrementable() -> Html {
    ///     let counter = use_state(|| 0);
    ///     let onclick = {
    ///         let counter = counter.clone();
    ///         Callback::from(move |_| counter.set(*counter + 1))
    ///     };
    ///     html! {
    ///         <div>
    ///             <p>{ format!("Value: {}", *counter) }</p>
    ///             <button {onclick}>{ "Add" }</button>
    ///         </div>
    ///     }
    /// }
    ///
    /// # use wasm_bindgen_test::wasm_bindgen_test;
    /// use frontest::{Query, HasText, HasRole};
    /// use frontest::yew::render;
    /// use yew::html;
    ///
    /// #[wasm_bindgen_test]
    /// async fn clicking_on_button_should_increment_value() {
    ///     let mount = render(html! { <Incrementable /> }).await;
    ///     let value = mount.get(&HasText("Value:")).unwrap();
    ///     let button = mount.get(&HasRole("button")).unwrap();
    ///
    ///     assert_eq!("Value: 0", value.inner_text());
    ///     button.click();
    ///     assert_eq!("Value: 1", value.inner_text());
    /// }
    /// ```
    ///
    /// [`html`]: ::yew::html!
    /// [`element`]: web_sys::Element
    pub async fn render(content: Html) -> Element {
        let div = gloo::utils::document().create_element("div").unwrap();
        gloo::utils::body().append_child(&div).unwrap();
        let res = div.clone();
        ::yew::start_app_with_props_in_element::<Wrapper>(div, WrapperProps { content });

        res
    }

    #[cfg(test)]
    #[test]
    fn doctest_yew_render() {
        use ::yew::prelude::*;
        #[function_component(Incrementable)]
        fn incrementable() -> Html {
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

        use crate::yew::render;
        use crate::{HasRole, HasText, Query};
        use ::yew::html;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn clicking_on_button_should_increment_value() {
            let mount = render(html! { <Incrementable /> }).await;
            let value = mount.get(&HasText("Value:")).unwrap();
            let button = mount.get(&HasRole("button")).unwrap();

            assert_eq!("Value: 0", value.inner_text());
            button.click();
            assert_eq!("Value: 1", value.inner_text());
        }
    }
}

/// Preempt execution of current task to let the js's main thread do things like re-render.
///
/// # Warning:
/// I'm currently unsure in which condition tick is required. Most cases should work without the need of using it.
#[doc(hidden)]
pub async fn tick() {
    sleep(Duration::ZERO).await;
}

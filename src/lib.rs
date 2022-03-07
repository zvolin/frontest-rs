//! Quick and lightweight library to query and assert DOM.
//!
//! Frontest is heavily inspired by [`dom-testing-library`] and [`react-testing-library`].
//! It provides a set of queries that you can use to quickly find your elements in document
//! with respect to accessibility priorities.
//!
//! # Basic usage:
//!
//! A [`Query`] trait allows for selecting elements in a way that users would interact with them
//! with the respect for assisstive technology.
//!
//! ```no_run
//! use frontest::query::*;
//! use gloo::utils::{body, document};
//!
//! let div = document().create_element("div").unwrap();
//! div.set_inner_html(
//!     r#"<div>
//!         <label for="best-language">Type rust</label>
//!         <input id="best-language" />
//!
//!         <textarea placeholder="rust rocks on frontend as..." />
//!     </div>"#
//! );
//! body().append_child(&div).unwrap();
//!
//! assert_eq!(
//!     div.get_all(
//!         &HasRole("textbox").and(HasLabel("Type rust").or(HasPlaceholder("rust rocks")))
//!     )
//!     .len(),
//!     2
//! );
//!
//! body().remove_child(&div).unwrap();
//! ```
//!
//! # About testing:
//!
//! This library aims to allow developers to test their application in a way that a user would interact with it.
//! For this purpose it is recommended to prioritize certain queries above another.
//! Currently only two matchers are implemented. More will be available in future releases.
//! Matchers should be prioritized like so:
//! - [`HasRole`] Should always be used where possible. It allows accessing elements that are exposed into accessibility tree.
//! - [`HasLabel`] Also should be used where possible. Is supported by screen readers and allows for easier focusing elements.
//! - [`HasPlaceholder`] Not as great option as predecessors, however still a better alternative than [`HasText`] for accessible elements.
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
//! # use gloo::utils::body;
//! use frontest::query::{Query, HasText, HasRole};
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
//!
//!     body().remove_child(&mount).unwrap();
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
//! [`HtmlElement`]: web_sys::HtmlElement
//! [`HasText`]: query::HasText
//! [`HasLabel`]: query::HasLabel
//! [`HasPlaceholder`]: query::HasPlaceholder
//! [`HasRole`]: query::HasRole
//! [`Not`]: query::Not
//! [`Query`]: query::Query
//! [`Joinable`]: query::Joinable
use gloo::timers::future::sleep;
use std::time::Duration;

#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

pub mod query;

#[cfg(test)]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn doctest_basic_usage() {
    use crate::query::*;
    use gloo::utils::{body, document};

    let div = document().create_element("div").unwrap();
    div.set_inner_html(
        r#"<div>
            <label for="best-language">Type rust</label>
            <input type="text" id="best-language" />
   
            <textarea placeholder="rust rocks on frontend as..." />
        </div>"#,
    );
    body().append_child(&div).unwrap();

    assert_eq!(
        div.get_all(
            &HasRole("textbox").and(HasLabel("Type rust").or(HasPlaceholder("rust rocks")))
        )
        .len(),
        2
    );

    body().remove_child(&div).unwrap();
}

/// A helpers when testing frontend made with [`yew`]
///
/// [`yew`]: ::yew
#[cfg(feature = "yew")]
pub mod yew {
    use ::yew::prelude::*;
    use web_sys::Element;

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
    /// # use gloo::utils::body;
    /// use frontest::query::{Query, HasText, HasRole};
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
    ///
    ///     body().remove_child(&mount).unwrap();
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
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn doctest_yew_render() {
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

        use crate::query::{HasRole, HasText, Query};
        use crate::yew::render;
        use ::yew::html;
        // use wasm_bindgen_test::wasm_bindgen_test;
        use gloo::utils::body;

        // #[wasm_bindgen_test]
        // async fn clicking_on_button_should_increment_value() {
        let mount = render(html! { <Incrementable /> }).await;
        let value = mount.get(&HasText("Value:")).unwrap();
        let button = mount.get(&HasRole("button")).unwrap();

        assert_eq!("Value: 0", value.inner_text());
        button.click();
        assert_eq!("Value: 1", value.inner_text());

        body().remove_child(&mount).unwrap();
        // }
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

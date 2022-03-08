/// Find various elements across the website as the user would.
use gloo::utils::document;
use wasm_bindgen::JsCast;
use web_sys::{
    Element, HtmlButtonElement, HtmlElement, HtmlInputElement, HtmlLabelElement, HtmlMeterElement,
    HtmlOutputElement, HtmlProgressElement, HtmlSelectElement, HtmlTextAreaElement,
};

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
/// # use gloo::utils::{body, document};
/// use frontest::prelude::*;
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
///
/// body().remove_child(&div).unwrap();
/// ```
pub trait Matcher {
    /// Returns `true` if the element was matched by [`Matcher`].
    fn matches(&self, elem: &HtmlElement) -> bool;
}

#[cfg(test)]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn doctest_matcher() {
    use crate::query::Matcher;
    use crate::query::{HasRole, Joinable, Query};
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

    body().remove_child(&div).unwrap();
}

/// Consumes a [`Matcher`] and returns a negation of it.
///
/// Utility wrapper that performs a logical `not` operation on a matcher.
///
/// # Example:
///
/// ```no_run
/// use gloo::utils::{body, document};
/// use frontest::prelude::*;

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
///
/// body().remove_child(&div).unwrap();
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
    use crate::query::{HasRole, HasText, Joinable, Not, Query};
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

    body().remove_child(&div).unwrap();
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
/// use frontest::prelude::*;
///
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
///
/// body().remove_child(&div).unwrap();
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
    use crate::query::{HasText, Query};
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

    body().remove_child(&div).unwrap();
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
/// use frontest::prelude::*;
///
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
///
/// body().remove_child(&div).unwrap();
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
    use crate::query::{HasRole, Query};
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

    body().remove_child(&div).unwrap();
}

/// Matches components that have given label.
///
/// This is also a great method for interacting with DOM in the way as a user would.
/// Labels are not only visually connected with elements but also programatically.
/// Screen readers will read out the labels when given component is selected as well as
/// clicking on a label results in selecting labeled component.
///
/// [`Labeling'] is supported for input elements (except type="hidden"), button, meter,
/// output, progress, select and text area.
///
/// # Example:
///
/// ```no_run
/// # use gloo::utils::{body, document};
/// use frontest::prelude::*;
///
/// let div = document().create_element("div").unwrap();
/// div.set_inner_html(
///     r#"<div>
///         <!-- for id attributes -->
///         <label for="best-language">Type rust</label>
///         <input id="best-language" />
///
///         <!-- implicit labels -->
///         <label>Type rust <meter /></label>
///
///         <!-- wrapped implicit labels -->
///         <label>
///           <span>Type rust</span>
///           <button />
///         </label>
///
///         <!-- aria-labelledby attributes -->
///         <label id="best-language">Type rust</label>
///         <input aria-labelledby="best-language" />
///
///         <!-- aria-label attributes are not supported as they are not visible to user -->
///         <input aria-label="Type rust" />
///     </div>"#,
/// );
/// body().append_child(&div).unwrap();
///
/// assert_eq!(div.get_all(&HasLabel("Type rust")).len(), 4);
///
/// body().remove_child(&div).unwrap();
/// ```
/// [`Labeling`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label
pub struct HasLabel<'a>(pub &'a str);

impl<'a> Matcher for HasLabel<'a> {
    fn matches(&self, elem: &HtmlElement) -> bool {
        // Check if element is one of types that support labeling
        // and if so, extract labels
        // https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label
        let labels = if let Some(elem) = elem.dyn_ref::<HtmlInputElement>() {
            // input type="hidden" doesn't support labels
            if elem.type_() == "hidden" {
                return false;
            }
            elem.labels().unwrap()
        } else if let Some(elem) = elem.dyn_ref::<HtmlButtonElement>() {
            elem.labels()
        } else if let Some(elem) = elem.dyn_ref::<HtmlMeterElement>() {
            elem.labels()
        } else if let Some(elem) = elem.dyn_ref::<HtmlOutputElement>() {
            elem.labels()
        } else if let Some(elem) = elem.dyn_ref::<HtmlProgressElement>() {
            elem.labels()
        } else if let Some(elem) = elem.dyn_ref::<HtmlSelectElement>() {
            elem.labels()
        } else if let Some(elem) = elem.dyn_ref::<HtmlTextAreaElement>() {
            elem.labels()
        } else {
            return false;
        };
        // Check if element is labeled by requested label
        if (0..labels.length())
            .filter_map(|idx| labels.get(idx))
            .any(|label| label.text_content().as_deref() == Some(self.0))
        {
            return true;
        }
        // Check if element is implicitly wrapped with label
        if let Some(parent) = elem.parent_element() {
            if let Some(label) = parent.dyn_ref::<HtmlLabelElement>() {
                let child_nodes = label.child_nodes();
                if (0..child_nodes.length())
                    .filter_map(|idx| child_nodes.get(idx))
                    .filter(|child| Some(elem) != child.dyn_ref())
                    .any(|child| child.text_content().as_deref().map(str::trim) == Some(self.0))
                {
                    return true;
                }
            }
        }
        // Check if element is aria-labelledby a label
        if let Some(label) = elem.get_attribute("aria-labelledby") {
            if document().get_element_by_id(&label).is_some() {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn doctest_has_label() {
    use crate::query::{HasLabel, Query};
    use gloo::utils::{body, document};

    let div = document().create_element("div").unwrap();
    div.set_inner_html(
        r#"<div>
            <!-- for id attributes -->
            <label for="best-language">Type rust</label>
            <input id="best-language" />

            <!-- implicit labels -->
            <label>Type rust <meter /></label>

            <!-- wrapped implicit labels -->
            <label>
              <span>Type rust</span>
              <button />
            </label>

            <!-- aria-labelledby attributes -->
            <label id="best-language">Type rust</label>
            <input aria-labelledby="best-language" />

            <!-- aria-label attributes are not supported as they are not visible to user -->
            <input aria-label="Type rust" />
        </div>"#,
    );
    body().append_child(&div).unwrap();

    assert_eq!(div.get_all(&HasLabel("Type rust")).len(), 4);

    body().remove_child(&div).unwrap();
}

/// Matches components that have given placeholder text.
///
/// Placeholders are not a substitute for labels. If placeholder is the only identifier
/// for an input, any assistive technology will not be able to identify them.
/// It is still a better fallback than just using [`HasText`] for accessible elements.
///
/// # Example:
///
/// ```no_run
/// # use gloo::utils::{body, document};
/// use frontest::prelude::*;
///
/// let div = document().create_element("div").unwrap();
/// div.set_inner_html(
///     r#"<div>
///         <button>tests rocks</button>
///         <input placeholder="tests rocks" />
///     </div>"#,
/// );
/// body().append_child(&div).unwrap();
///
/// assert_eq!(
///     div.get(&HasPlaceholder("tests")).unwrap().tag_name(),
///     "INPUT"
/// );
///
/// body().remove_child(&div).unwrap();
/// ```
pub struct HasPlaceholder<'a>(pub &'a str);

impl<'a> Matcher for HasPlaceholder<'a> {
    fn matches(&self, elem: &HtmlElement) -> bool {
        let placeholder = if let Some(elem) = elem.dyn_ref::<HtmlInputElement>() {
            elem.placeholder()
        } else if let Some(elem) = elem.dyn_ref::<HtmlTextAreaElement>() {
            elem.placeholder()
        } else {
            return false;
        };
        placeholder.contains(self.0)
    }
}

#[cfg(test)]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn doctest_has_placeholder() {
    use crate::query::{HasPlaceholder, Query};
    use gloo::utils::{body, document};

    let div = document().create_element("div").unwrap();
    div.set_inner_html(
        r#"<div>
            <button>tests rocks</button>
            <input placeholder="tests rocks" />
        </div>"#,
    );
    body().append_child(&div).unwrap();

    assert_eq!(
        div.get(&HasPlaceholder("tests")).unwrap().tag_name(),
        "INPUT"
    );

    body().remove_child(&div).unwrap();
}

/// A trait for joining multiple matchers.
///
/// It is automatically implemented for all matchers.
/// It allows for joining matchers using `or` and `and` methods that consume both matchers
/// and returns a joined matcher. It can be chained with multiple calls.
///
/// # Example:
/// ```no_run
/// use gloo::utils::{body, document};
/// use frontest::prelude::*;
///
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
///
/// body().remove_child(&div).unwrap();
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
    use crate::query::{HasRole, HasText, Joinable, Query};
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

    body().remove_child(&div).unwrap();
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

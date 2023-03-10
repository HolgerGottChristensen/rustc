// @has issue_41783/struct.Foo.html
// @!hasraw - 'space'
// @!hasraw - 'comment'
// @hasraw - '<span class="attr">#[outer]'
// @!hasraw - '<span class="attr">#[outer]</span>'
// @hasraw - '#![inner]</span>'
// @!hasraw - '<span class="attr">#![inner]</span>'
// @snapshot 'codeblock' - '//*[@class="rustdoc-toggle top-doc"]/*[@class="docblock"]//pre/code'

/// ```no_run
/// # # space
/// # comment
/// ## single
/// ### double
/// #### triple
/// ##[outer]
/// ##![inner]
/// ```
pub struct Foo;

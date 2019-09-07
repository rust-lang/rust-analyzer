export default `use crate::List::*;

mod unresolved_module;

enum List {
    // Cons: Tuple struct that wraps an element and a pointer to the next node
    Cons(u32, Box<List>),
    // Nil: A node that signifies the end of the linked list
    Nil,
}

// Methods can be attached to an enum
impl List {
    /// Create an empty list
    fn new() -> List {
        // \`Nil\` has type \`List\`
        Nil
    }

    /// Consume a list, and return the same list with a new element at its front
    fn prepend(self, elem: u32) -> List {
        // \`Cons\` also has type List
        Cons(elem, Box::new(self))
    }

    /// Return the length of the list
    fn len(&self) -> u32 {
        // \`self\` has to be matched, because the behavior of this method
        // depends on the variant of \`self\`
        // \`self\` has type \`&List\`, and \`*self\` has type \`List\`, matching on a
        // concrete type \`T\` is preferred over a match on a reference \`&T\`
        match *self {
            // Can't take ownership of the tail, because \`self\` is borrowed;
            // instead take a reference to the tail
            Cons(_, ref tail) => 1 + tail.len(),
            // Base Case: An empty list has zero length
            Nil => 0
        }
    }

    /// Return representation of the list as a (heap allocated) string
    fn stringify(&self) -> String {
        match *self {
            Cons(head, ref tail) => {
                // \`format!\` is similar to \`print!\`, but returns a heap
                // allocated string instead of printing to the console
                format!("{}, {}", head, tail.stringify())
            },
            Nil => {
                format!("Nil")
            },
        }
    }
}


/// \`\`\`rust
/// fn main_in_comment() {
///     // This binding lives in the main function
///     let long_lived_binding = 1;
/// 
///     // This is a block, and has a smaller scope than the main function
///     {
///         // This binding only exists in this block
///         let short_lived_binding = 2;
/// 
///         println!("inner short: {}", short_lived_binding);
/// 
///         // This binding *shadows* the outer one
///         let long_lived_binding = 5_f32;
/// 
///         println!("inner long: {}", long_lived_binding);
///     }
///     // End of the block
/// 
///     // Error! \`short_lived_binding\` doesn't exist in this scope
///     println!("outer short: {}", short_lived_binding);
///     // FIXME ^ Comment out this line
/// 
///     println!("outer long: {}", long_lived_binding);
///     
///     // This binding also *shadows* the previous binding
///     let long_lived_binding = 'a';
///     
///     println!("outer long: {}", long_lived_binding);
/// }
/// \`\`\`
fn main() {
    // Create an empty linked list
    let mut list = List::new();

    // Prepend some elements
    list = list.prepend(1);
    list = list.prepend(2);
    list = list.prepend(3);

    // Show the final state of the list
    println!("linked list has length: {}", list.len());
    println!("{}", list.stringify());
}
`;

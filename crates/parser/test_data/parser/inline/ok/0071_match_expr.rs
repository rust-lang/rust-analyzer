fn foo() {
    match () { };
    match S {};
    match { } { _ => () };
    match { S {} } {};

    S.match { _ => () };
    S.match { }.match { };
    1.match { };
    x.0.match { };
    x.0().match { }?.hello();
    x.0.0.match { };
    x.0. match { };
}

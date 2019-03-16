fn main() {
    #[cfg(test)]
    ();

    #[cfg(test)]
    (1 + 2);

    #[cfg(test)]
    main();

    #[cfg(test)]
    ().clone();

    #[cfg(test)]
    println!("hello");

    while false {
        #[cfg(test)]
        continue;

        #[cfg(test)]
        break;

        #[cfg(test)]
        return;
    }

    #[cfg(test)]
    {}

    #[cfg(test)]
    unsafe {}

    #[cfg(test)]
    while false {}

    #[cfg(test)]
    for _ in 0..10 {}

    #[cfg(test)]
    loop {}

    #[cfg(test)]
    match 0 {
        _ => {}
    }
}

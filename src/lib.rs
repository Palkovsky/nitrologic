#![allow(unused_imports)]

pub mod set;
pub mod plot;
pub mod fuzz;
pub mod defuzz;
pub mod common;

use set::*;
use fuzz::*;
use common::*;

#[test]
fn test_call(
) -> FuzzyResult<()> {
    let set = fuzzy! {
        "very quiet" => (0.0, 1.0), (10.0, 1.0), (20.0, 0.5), (30.0, 0.0);
        "quiet" => (10.0, 0.0), (20.0, 0.5), (30.0, 1.0), (40.0, 1.0), (50.0, 0.5), (60.0, 0.0);
        "loud" => (40.0, 0.0), (50.0, 0.5), (60.0, 1.0), (70.0, 1.0), (80.0, 0.5), (90.0, 0.0);
        "very loud" => (70.0, 0.0), (80.0, 0.5), (90.0, 1.0), (100.0, 1.0);
    }?;

    assert_eq!(set.call(20.0)?.remove("very quiet"), Some(0.5));
    assert_eq!(set.call(20.0)?.remove("quiet"), Some(0.5));
    assert_eq!(set.call(30.0)?.remove("very quiet"), Some(0.0));
    assert_eq!(set.call(-30.0)?.remove("very quiet"), Some(1.0));
    assert_eq!(set.call(130.0)?.remove("very quiet"), Some(0.0));
    assert_eq!(set.call(130.0)?.remove("very loud"), Some(1.0));
    assert_eq!(set.call(130.0)?.remove("loud"), Some(0.0));
    assert_eq!(set.call(90.0)?.remove("loud"), Some(0.0));
    assert_eq!(set.call(90.0)?.remove("very loud"), Some(1.0));
    assert_eq!(set.call(25.0)?.remove("very quiet"), Some(0.25));

    plot::set(&set, "Loudness", "Y").to_svg("test1.svg")
}

#[test]
fn test_threshold(
) -> FuzzyResult<()> {
    Ok(())
}

#[test]
fn test_fuzzer(
) -> FuzzyResult<()> {
    // Construct model
    let fuzzer = Fuzzer::new(
    ).fuzzify(
        "loudness",
        fuzzy! {
            "very quiet" => (0.0, 1.0), (10.0, 1.0), (20.0, 0.5), (30.0, 0.0);
            "quiet"      => (10.0, 0.0), (20.0, 0.5), (30.0, 1.0), (40.0, 1.0), (50.0, 0.5), (60.0, 0.0);
            "loud"       => (40.0, 0.0), (50.0, 0.5), (60.0, 1.0), (70.0, 1.0), (80.0, 0.5), (90.0, 0.0);
            "very loud"  => (70.0, 0.0), (80.0, 0.5), (90.0, 1.0), (100.0, 1.0);
        }?
    ).fuzzify(
        "tod",
        fuzzy! {
            "morning" => (1.0, 0.0), (3.0, 0.5), (5.0, 1.0), (7.0, 1.0), (9.0, 0.5), (11.0, 0.0);
            "noon"    => (7.0, 0.0), (9.0, 0.50), (11.0, 1.0), (13.0, 1.0), (15.0, 0.50), (17.0, 0.0);
            "evening" => (13.0, 0.0), (15.0, 0.50), (17.0, 1.0), (19.0, 1.0), (21.0, 0.50), (23.0, 0.0);
            "night"   => (0.0, 1.0), (1.0, 1.0), (3.0, 0.5), (5.0, 0.0), (19.0, 0.0), (21.0, 0.5), (23.0, 1.0);
        }?
    ).defuzzify(
        "change",
        fuzzy! {
            "vol down" => (0.0, 1.0), (2.0, 1.0), (3.0, 0.5), (4.0, 0.0), (7.0, 0.0);
            "keep"     => (2.0, 0.0), (3.0, 0.5), (4.0, 1.0), (6.0, 1.0), (7.0, 0.5), (8.0, 0.0);
            "vol up"   => (3.0, 0.0), (6.0, 0.0), (7.0, 0.5), (8.0, 1.0), (10.0, 1.0);
        }?
    // Good Moaning
    ).rule(and! {"loudness" => "very quiet", "tod" => "morning"; "change" => "vol up" }
    ).rule(and! {"loudness" => "quiet", "tod" => "morning"; "change" => "keep" }
    ).rule(and! {"loudness" => "loud", "tod" => "morning"; "change" => "keep" }
    ).rule(and! {"loudness" => "very loud", "tod" => "morning"; "change" => "vol down" }
    // Noon
    ).rule(and! {"loudness" => "very quiet", "tod" => "noon"; "change" => "vol up" }
    ).rule(and! {"loudness" => "quiet", "tod" => "noon"; "change" => "vol up" }
    ).rule(and! {"loudness" => "loud", "tod" => "noon"; "change" => "keep" }
    ).rule(and! {"loudness" => "very loud", "tod" => "noon"; "change" => "vol down" }
    // Evening
    ).rule(and! {"loudness" => "very quiet", "tod" => "evening"; "change" => "vol up" }
    ).rule(and! {"loudness" => "quiet", "tod" => "evening"; "change" => "keep" }
    ).rule(and! {"loudness" => "loud", "tod" => "evening"; "change" => "vol down" }
    ).rule(and! {"loudness" => "very loud", "tod" => "evening"; "change" => "vol down" }
    // Night
    ).rule(and! {"loudness" => "very quiet", "tod" => "night"; "change" => "vol up" }
    ).rule(and! {"loudness" => "quiet", "tod" => "night"; "change" => "keep" }
    ).rule(and! {"loudness" => "loud", "tod" => "night"; "change" => "vol down" }
    ).rule(and! {"loudness" => "very loud", "tod" => "night"; "change" => "vol down" });

    // Specify inputs
    let input = values! { "loudness" => 60.0; String::from("tod") => 21.0 };
    // Calculate output set
    let change = fuzzer.apply(&input)?
        .remove("change")
        .unwrap();
    // Apply defuzzification method
    let cog = defuzz::cog(change.points("out")?)?;
    plot::set(&change, format!("COG: {:.2}", cog), "Y").to_svg("test3.svg")?;

    println!("COG: {:?}", cog) ;
    println!("{:?}", change);

    assert!(false);
    Ok(())
}

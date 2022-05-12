#![cfg(test)]
#![allow(dead_code)]

use crate::grouped_collections::GroupedCollection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Animal {
    Beaver,
    Cat,
    Horse,
    Donkey,
}
use Animal::*;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Foot {
    Claw,  // Pairs with Beaver, Cat
    Hoof,  // Pairs with Horse, Donkey
    Talon, // Does not pair (empty group)
}
pub use Foot::*;

pub fn verify_grouped_collection<Map>(mut map: Map)
where
    Map: for<'a> GroupedCollection<'a, Foot, Animal, Vec<Animal>> + Clone,
{
    #[rustfmt::skip]
    let claw_pairs = [
        (Claw, Beaver),
        (Claw, Cat),
        (Claw, Beaver),
    ];

    #[rustfmt::skip]
    let hoof_pairs = [
        (Hoof, Horse),
        (Hoof, Donkey),
    ];

    // Just to try to provide some additional safety checks against strange lifetime issues,
    // we'll run our tests against an owned map as well as some references and smart pointers.
    let refmap: &mut Map = &mut map.clone();
    let mut boxmap: Box<Map> = Box::new(map.clone());

    // The following does not work, hence the limitations section in the docs.
    // let mut boxdynmap: Box<dyn GroupedCollection<_, _, _, Iter = _>> = Box::new(map.clone());

    // Use add(); tests below will verify that it worked.
    for (k, v) in &claw_pairs {
        map.add(k.clone(), v.clone());
        refmap.add(k.clone(), v.clone());
        boxmap.add(k.clone(), v.clone());
    }
    for (k, v) in &hoof_pairs {
        map.add(k.clone(), v.clone());
        refmap.add(k.clone(), v.clone());
        boxmap.add(k.clone(), v.clone());
    }

    let claws = claw_pairs
        .iter()
        .map(|(_, v)| v.clone())
        .collect::<Vec<Animal>>();

    let hooves = hoof_pairs
        .iter()
        .map(|(_, v)| v.clone())
        .collect::<Vec<Animal>>();

    // Check get()
    assert_eq!(map.get(&Claw), Some(&claws));
    assert_eq!(map.get(&Hoof), Some(&hooves));
    assert_eq!(map.get(&Talon), None);

    assert_eq!(refmap.get(&Claw), Some(&claws));
    assert_eq!(refmap.get(&Claw), Some(&claws));
    assert_eq!(refmap.get(&Claw), Some(&claws));

    assert_eq!(boxmap.get(&Hoof), Some(&hooves));
    assert_eq!(boxmap.get(&Talon), None);
    assert_eq!(boxmap.get(&Hoof), Some(&hooves));

    // Check iter()
    let pairs = map.iter().collect::<Vec<_>>();
    assert!(pairs.contains(&(&Claw, &vec![Beaver, Cat, Beaver])));
    assert!(pairs.contains(&(&Hoof, &vec![Horse, Donkey])));

    let pairs = refmap.iter().collect::<Vec<_>>();
    assert!(pairs.contains(&(&Claw, &vec![Beaver, Cat, Beaver])));
    assert!(pairs.contains(&(&Hoof, &vec![Horse, Donkey])));

    let pairs = boxmap.iter().collect::<Vec<_>>();
    assert!(pairs.contains(&(&Claw, &vec![Beaver, Cat, Beaver])));
    assert!(pairs.contains(&(&Hoof, &vec![Horse, Donkey])));

    // There have been some weird lifetime issues with using add() after get() with references;
    // let's just quickly make sure it works here.
    map.add(Claw, Beaver);
    refmap.add(Claw, Beaver);
    boxmap.add(Claw, Beaver);
}

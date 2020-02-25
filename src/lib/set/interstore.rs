use super::super::domain::{Data, Primitive};
use super::super::errors::{ApplicationError, Fallible, Flattenable};
use super::utilities::get_set;
use std::collections::{HashMap, HashSet};

pub fn command(
  store: &mut HashMap<String, Data>,
  destination: &str,
  base_key: &str,
  keys: &[String],
) -> Result<usize, ApplicationError> {
  let base = get_set(store, base_key)?;
  let mut result: HashSet<Primitive> = HashSet::new();
  let mut iterated = keys.iter().map(|key| get_set(&store, key));
  let first = iterated.next().fail_to("No sets to intersect").flatten()?;
  iterated
    .fold(Ok(vec![first]), |acc, next| match (acc, next) {
      (Err(e), _) | (_, Err(e)) => Err(e),
      (Ok(mut acc), Ok(a)) => {
        acc.push(a);
        Ok(acc)
      }
    })?
    .iter()
    .for_each(|set| {
      set.iter().for_each(|el| {
        if base.contains(el) {result.insert(el)};
      })
    });
  let size = result.len();
  store.insert(destination.to_string(), result.into());
  Ok(size)
}

#[cfg(test)]
mod test {
  use super::super::super::domain::Primitive;
  use super::{command, Data};
  use proptest::collection::hash_set;
  use proptest::prelude::*;
  use proptest::string::{string_regex, RegexGeneratorStrategy};
  use std::collections::{HashMap, HashSet};

  fn valid_keys() -> RegexGeneratorStrategy<String> {
    match string_regex("[^\\s]+") {
      Ok(s) => s,
      Err(_) => panic!("strategy failed"),
    }
  }

  proptest! {
      #[test]
      fn intersection_is_idempotent(
        dest in valid_keys(),
        snd_dest in valid_keys(),
        a in (valid_keys(), hash_set(any::<Primitive>(), 1..100)),
        b in (valid_keys(), hash_set(any::<Primitive>(), 1..100))
      ) {
          let mut store: HashMap<String, Data> = HashMap::new();
          let (a_key, a_set) = a;
          let (b_key, b_set) = b;
          store.insert(a_key.clone(), a_set.into());
          store.insert(b_key.clone(), b_set.into());
          command(&mut store, &dest, &a_key, &vec![b_key.clone()][..])?;
          command(&mut store, &snd_dest, &dest, &vec![b_key][..])?;
          assert_eq!(store.get(&dest), store.get(&snd_dest))
      }
  }

  proptest! {
      #[test]
      fn disjoint_intersection_is_empty(
        dest in valid_keys(),
        a in (valid_keys(), hash_set(any::<Primitive>(), 1..100)),
        b in (valid_keys(), hash_set(any::<Primitive>(), 1..100))
      ) {
          let mut store: HashMap<String, Data> = HashMap::new();
          let (a_key, a_set) = a;
          let (b_key, b_set) = b;
          store.insert(a_key.clone(), a_set.difference(&b_set).cloned().collect::<HashSet<Primitive>>().into());
          store.insert(b_key.clone(), b_set.into());
          command(&mut store, &dest, &a_key, &vec![b_key.clone()][..])?;
          if let Some(Data::Set(set)) = store.get(&dest) {assert_eq!(set.len(), 0)} else { panic!("something odd found at key")}
      }
  }

  proptest! {
      #[test]
      fn self_intersection_is_self(
        dest in valid_keys(),
        a in (valid_keys(), hash_set(any::<Primitive>(), 1..100))
      ) {
          let mut store: HashMap<String, Data> = HashMap::new();
          let (a_key, a_set) = a;
          let a_len = a_set.len();
          store.insert(a_key.clone(), a_set.into());
          command(&mut store, &dest, &a_key.clone(), &vec![a_key][..])?;
          let result = store.get(&dest);
          if let Some(Data::Set(set)) = result {assert_eq!(set.len(), a_len)} else { panic!("something odd found at key")}
      }
  }

  proptest! {
      #[test]
      fn returns_size_of_dest(
        dest in valid_keys(),
        a in (valid_keys(), hash_set(any::<Primitive>(), 1..100)),
        b in (valid_keys(), hash_set(any::<Primitive>(), 1..100))
      ) {
          let mut store: HashMap<String, Data> = HashMap::new();
          let (a_key, a_set) = a;
          let (b_key, b_set) = b;
          store.insert(a_key.clone(), a_set.into());
          store.insert(b_key.clone(), b_set.into());
          let resulting_size = command(&mut store, &dest, &a_key, &vec![b_key][..])?;
          let resulting_set = store.get(&dest);
          if let Some(Data::Set(set)) = resulting_set {assert_eq!(set.len(), resulting_size)} else { panic!("something odd found at key")}
      }
  }
}
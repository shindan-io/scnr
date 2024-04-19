// https://fasterthanli.me/articles/recursive-iterators-rust

struct Node {
  values: Vec<i32>,
  children: Vec<Node>,
}

impl Node {
  pub fn values<'a>(&'a self) -> Box<dyn Iterator<Item = &i32> + 'a> {
    Box::new(self.values.iter().chain(self.children.iter().map(|n| n.values()).flatten()))
  }
}

// struct NodeIter<'a> {
//   viter: Box<dyn Iterator<Item = &'a i32> + 'a>,
//   citer: Box<dyn Iterator<Item = &'a Node> + 'a>,
// }
// impl<'a> Iterator for NodeIter<'a> {
//   type Item = &'a i32;

//   fn next(&mut self) -> Option<Self::Item> {
//     // if we still have some values of our own to yield...
//     if let Some(val) = self.viter.next() {
//       // then yield them
//       Some(val)
//     } else {
//       // if we're out of values, but we still have children to walk...
//       if let Some(child) = self.citer.next() {
//         // then yield all their values.
//         self.viter = Box::new(child.values());
//         // call next() again, hopefully immediately yielding the
//         // branch's next value, but if not, we'll just keep recursing
//         // until we find a non-empty branch or run out of nodes.
//         self.next()
//       } else {
//         None
//       }
//     }
//   }
// }

// impl Node {
//   fn values<'a>(&'a self) -> NodeIter<'a> {
//     NodeIter {
//       // does not compile
//       viter: Box::new(self.values.iter()),
//       citer: Box::new(self.children.iter()),
//     }
//   }
// }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test() {
    let root = Node {
      values: vec![1, 2, 3],
      children: vec![
        Node { values: vec![4, 5], children: vec![Node { values: vec![6], children: vec![] }] },
        Node { values: vec![7], children: vec![] },
      ],
    };

    let iterated = root.values().cloned().collect::<Vec<_>>();
    assert_eq!(iterated, vec![1, 2, 3, 4, 5, 6, 7]);
  }
}

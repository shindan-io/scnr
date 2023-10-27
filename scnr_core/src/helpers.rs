// use super::*;
// use async_stream::stream;

// pub fn to_scan_stream<T>(nodes: Vec<T>) -> ScanStream
// where
//   T: Node + 'static,
// {
//   let stream = stream! {
//     for node in nodes {
//       yield Ok(Box::pin(node) as  BoxedNode);
//     }
//   };
//   Box::pin(stream)
// }

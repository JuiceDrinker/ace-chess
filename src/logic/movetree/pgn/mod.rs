pub mod parser;
// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test]
//     fn it_move_entries() {
//         let mut parser = Parser::new();
//         let _ = parser.move_entry("1.d4 e5").unwrap();
//
//         assert_eq!(parser.graph.0.count(), 2);
//     }
//
//     #[test]
//     fn it_move_entries_without_move_number() {
//         let mut parser = Parser::new();
//         let _ = parser.move_entry("d4?").unwrap();
//         assert_eq!(parser.graph.0.count(), 2);
//     }
//
//     #[test]
//     fn parses_nested_variations() {
//         let parser = Parser::new();
//         let res = parser
//             .parse("1. d4 ( 1. e4 e5 (2... Nf6 3. Nh3) ) d5 2.Nf3 (2. a4) 1-0")
//             // .parse("1. d4 ( 1. e4 e5 2. Nf3 Nf6 (2... a6 (2... b6 )(2... Nc6) 3. Na3)  ) d5 1-0")
//             // .parse("1.d4 e5 (1...e6 2.e4)")
//             .unwrap();
//
//         assert_eq!(res.0.count(), 9);
//     }
//
//     #[test]
//     #[should_panic]
//     fn should_panic_for_invalid_pgn() {
//         let pgn = Parser::new();
//         let _ = pgn.parse("1.z@2").unwrap();
//     }
//
//     #[test]
//     fn parses_to_linear_graph() {
//         let pgn = Parser::new();
//         let res = pgn.parse("1.e4 e5 2.d4 d5 1-0 ").unwrap();
//         assert_eq!(res.0.count(), 5);
//     }
//
//     #[test]
//     fn game_with_nested_comment_and_variations() {
//         let pgn = Parser::new();
//         let res = pgn
//             .parse("1.e4 {This is a comment}(1. d4 Nf6 ) e5 1-0")
//             .unwrap();
//
//         assert_eq!(res.0.count(), 5);
//     }
//
//     #[test]
//     fn debug() {
//         let pgn = Parser::new();
//         let res = pgn
//             .parse(
//                 "1. e4 e5 2. Nf3 Nc6 3. Bb5 {This opening is called the Ruy Lopez.} 3... a6
//                      4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
//     11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5
//                 Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6
//                 23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5
//                 hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5
//                 35. Ra7 g6 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6
//                 Nf2 42. g4 Bd3 43. Re6
//         1/2-1/2",
//             )
//             .unwrap();
//         assert_eq!(res.0.count(), 5);
//     }
//
//     #[test]
//     fn next_fen() {
//         let res =
//             Parser::generate_next_fen(&STARTING_POSITION_FEN.to_string(), &String::from("d4"));
//         assert_eq!(
//             res,
//             "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1"
//         );
//     }
//     // #[test]
//     // fn debug_fen() {
//     //     let res = Parser::generate_next_fen(
//     //         &String::from("r1bq1rk1/3nbppp/p1pp1n2/1P2p3/3PP3/1B3N1P/PP3PP1/RNBQR1K1 b - - 0 12"),
//     //         &String::from("axb5"),
//     //     );
//     // }
// }
//
// // impl<'a> From<AdjacencyList<'a>> for MoveTree {
// //     fn from(graph: AdjacencyList<'a>) -> Self {
// //         let mut tree = MoveTree::new();
// //         for (key, (entry, edges)) in &graph.0 {
// //             if let ListEntry::Node((move_text, _, _), fen, parent) = entry {
// //                 let node = TreeNode {
// //                     notation: (*move_text).to_string(),
// //                     fen: fen.to_string(),
// //                 };
// //                 tree.0.new_node(node);
// //                 for edge in
// //             };
// //         }
// //         // for edge in par
// //         tree
// //     }
// // }

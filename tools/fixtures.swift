import Foundation
import TrieDictionary
var trie=TrieDictionary<Int>()
for i in 0..<100 {
 let key="\(i % 7)-\(i % 11)"
 if i % 3 == 0 { _ = trie.removeValue(forKey:key) } else { trie[key]=i }
 print(trie.keys().sorted().map { "\($0):\(trie[$0]!)" }.joined(separator:","))
}

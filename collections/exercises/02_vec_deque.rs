#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 02: VecDeque で配送待ち行列を作る
//!
//! 配送依頼は通常、到着した順番で処理します。一方、安全確認などの緊急依頼は列の
//! 先頭へ追加し、現在の先頭依頼を延期するときは末尾へ回します。先頭と末尾の両方を
//! 効率よく操作できる `VecDeque<T>` で `DispatchQueue` を完成させてください。
//!
//! 仕様:
//! - 通常依頼は末尾へ追加し、先頭から FIFO 順に配送する。
//! - 緊急依頼は先頭へ追加する。緊急依頼が続いた場合は、最新の依頼を先に処理する。
//! - `next` は次の依頼を借用するだけで、待ち行列から取り出さない。
//! - `cancel_back` は最後尾の依頼を取り消し、その所有権を返す。
//! - `defer_next` は先頭を末尾へ移す。空なら `false`、1 件以上なら `true` を返す。
//! - 1 件だけの待ち行列を延期しても、内容は変わらない。
//!
//! ヒント:
//! - `push_back`、`push_front`、`front`、`pop_front`、`pop_back` を使います。
//! - `VecDeque` の内部要素が常に 1 つの連続スライスになるとは限りません。添字や
//!   内部配置に頼らず、待ち行列として操作してください。

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeliveryRequest {
    id: String,
    destination: String,
}

impl DeliveryRequest {
    fn new(id: &str, destination: &str) -> Self {
        Self {
            id: id.to_string(),
            destination: destination.to_string(),
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn destination(&self) -> &str {
        &self.destination
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct DispatchQueue {
    requests: VecDeque<DeliveryRequest>,
}

impl DispatchQueue {
    fn new() -> Self {
        todo!("空の VecDeque を持つ配送待ち行列を作ってください")
    }

    fn len(&self) -> usize {
        todo!("配送待ちの依頼数を返してください")
    }

    fn is_empty(&self) -> bool {
        todo!("配送待ち行列が空か確認してください")
    }

    fn enqueue(&mut self, request: DeliveryRequest) {
        todo!("通常依頼 {} を列の末尾へ追加してください", request.id())
    }

    fn preempt(&mut self, urgent: DeliveryRequest) {
        todo!("緊急依頼 {} を列の先頭へ追加してください", urgent.id())
    }

    fn next(&self) -> Option<&DeliveryRequest> {
        todo!("次に配送する依頼を取り出さずに借用してください")
    }

    fn dispatch_next(&mut self) -> Option<DeliveryRequest> {
        todo!("列の先頭から次の依頼を取り出してください")
    }

    fn cancel_back(&mut self) -> Option<DeliveryRequest> {
        todo!("列の最後尾から依頼を取り消してください")
    }

    fn defer_next(&mut self) -> bool {
        todo!("先頭の依頼があれば末尾へ移してください")
    }
}

fn main() {
    let mut queue = DispatchQueue::new();
    queue.enqueue(DeliveryRequest::new("REQ-401", "図書館"));
    queue.enqueue(DeliveryRequest::new("REQ-402", "研究棟"));
    queue.preempt(DeliveryRequest::new("SAFE-01", "安全確認地点"));

    println!("次の依頼: {:?}", queue.next());
    println!("配送開始: {:?}", queue.dispatch_next());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: &str, destination: &str) -> DeliveryRequest {
        DeliveryRequest::new(id, destination)
    }

    fn ids(queue: &DispatchQueue) -> Vec<&str> {
        queue.requests.iter().map(DeliveryRequest::id).collect()
    }

    #[test]
    fn 空の待ち行列を作る() {
        let queue = DispatchQueue::new();

        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        assert_eq!(queue.next(), None);
    }

    #[test]
    fn 通常依頼をfifo順に配送する() {
        let mut queue = DispatchQueue::new();
        queue.enqueue(request("REQ-401", "図書館"));
        queue.enqueue(request("REQ-402", "研究棟"));
        queue.enqueue(request("REQ-403", "学生寮"));

        assert_eq!(queue.len(), 3);
        assert_eq!(queue.dispatch_next(), Some(request("REQ-401", "図書館")));
        assert_eq!(queue.dispatch_next(), Some(request("REQ-402", "研究棟")));
        assert_eq!(queue.dispatch_next(), Some(request("REQ-403", "学生寮")));
        assert_eq!(queue.dispatch_next(), None);
        assert!(queue.is_empty());
    }

    #[test]
    fn 最新の緊急依頼を最優先する() {
        let mut queue = DispatchQueue::new();
        queue.enqueue(request("REQ-410", "図書館"));
        queue.enqueue(request("REQ-411", "研究棟"));
        queue.preempt(request("SAFE-01", "北門の安全確認"));
        queue.preempt(request("SAFE-02", "南門の安全確認"));

        assert_eq!(
            ids(&queue),
            vec!["SAFE-02", "SAFE-01", "REQ-410", "REQ-411"]
        );
        assert_eq!(
            queue.dispatch_next(),
            Some(request("SAFE-02", "南門の安全確認"))
        );
    }

    #[test]
    fn 次の依頼を取り出さずに借用する() {
        let mut queue = DispatchQueue::new();
        queue.enqueue(request("REQ-420", "工学部Ａ棟"));

        let next = queue.next().expect("依頼が存在する");
        assert_eq!(next.id(), "REQ-420");
        assert_eq!(next.destination(), "工学部Ａ棟");
        assert_eq!(queue.len(), 1);
        assert_eq!(ids(&queue), vec!["REQ-420"]);
    }

    #[test]
    fn 先頭を末尾へ延期する() {
        let mut queue = DispatchQueue::new();
        queue.enqueue(request("REQ-430", "図書館"));
        queue.enqueue(request("REQ-431", "研究棟"));
        queue.enqueue(request("REQ-432", "食堂"));

        assert!(queue.defer_next());
        assert_eq!(ids(&queue), vec!["REQ-431", "REQ-432", "REQ-430"]);
    }

    #[test]
    fn 空と一件の待ち行列を安全に延期する() {
        let mut queue = DispatchQueue::new();
        assert!(!queue.defer_next());
        assert!(queue.is_empty());

        queue.enqueue(request("REQ-440", "保健センター"));
        assert!(queue.defer_next());
        assert_eq!(ids(&queue), vec!["REQ-440"]);
    }

    #[test]
    fn 最後尾の依頼を取り消す() {
        let mut queue = DispatchQueue::new();
        assert_eq!(queue.cancel_back(), None);

        queue.enqueue(request("REQ-450", "図書館"));
        queue.enqueue(request("REQ-451", "体育館"));

        assert_eq!(queue.cancel_back(), Some(request("REQ-451", "体育館")));
        assert_eq!(ids(&queue), vec!["REQ-450"]);
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn 前後の操作を交互に行っても論理順序を保つ() {
        let mut queue = DispatchQueue::new();
        queue.enqueue(request("REQ-A", "図書館"));
        queue.enqueue(request("REQ-B", "研究棟"));
        queue.enqueue(request("REQ-C", "学生寮"));

        assert_eq!(queue.dispatch_next(), Some(request("REQ-A", "図書館")));
        queue.enqueue(request("REQ-D", "食堂"));
        queue.preempt(request("SAFE-X", "倉庫入口⚠️"));
        assert!(queue.defer_next());
        assert_eq!(ids(&queue), vec!["REQ-B", "REQ-C", "REQ-D", "SAFE-X"]);

        assert_eq!(queue.dispatch_next(), Some(request("REQ-B", "研究棟")));
        queue.enqueue(request("REQ-E", "工学部Ａ棟"));
        assert_eq!(queue.cancel_back(), Some(request("REQ-E", "工学部Ａ棟")));

        assert_eq!(queue.dispatch_next(), Some(request("REQ-C", "学生寮")));
        assert_eq!(queue.dispatch_next(), Some(request("REQ-D", "食堂")));
        assert_eq!(queue.dispatch_next(), Some(request("SAFE-X", "倉庫入口⚠️")));
        assert_eq!(queue.dispatch_next(), None);
    }
}

//! 問題 02 の解答例。

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
        Self {
            requests: VecDeque::new(),
        }
    }

    fn len(&self) -> usize {
        self.requests.len()
    }

    fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    fn enqueue(&mut self, request: DeliveryRequest) {
        self.requests.push_back(request);
    }

    fn preempt(&mut self, urgent: DeliveryRequest) {
        self.requests.push_front(urgent);
    }

    fn next(&self) -> Option<&DeliveryRequest> {
        self.requests.front()
    }

    fn dispatch_next(&mut self) -> Option<DeliveryRequest> {
        self.requests.pop_front()
    }

    fn cancel_back(&mut self) -> Option<DeliveryRequest> {
        self.requests.pop_back()
    }

    fn defer_next(&mut self) -> bool {
        let Some(request) = self.requests.pop_front() else {
            return false;
        };

        self.requests.push_back(request);
        true
    }
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

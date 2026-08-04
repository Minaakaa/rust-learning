#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 05: `Any`で拡張registryの型を安全に復元する
//!
//! `ExtensionRegistry`は異なる拡張をtrait objectとして所有し、exact concrete typeの
//! `TypeId`をkeyにして各型を1個ずつ保持します。共通の`status`はdyn dispatchで呼び、
//! 型固有の設定変更や所有値の回収だけをdowncastで行ってください
//!
//! 仕様:
//! - `Extension`は`Any + Send + Sync`をsupertraitにする
//! - `HashMap<TypeId, Box<dyn Extension + Send + Sync>>`で値を所有する
//! - `insert`は同じexact typeの以前の値を具体型の`Box`で返す
//! - `get`と`get_mut`はtrait upcasting後にborrowをdowncastする
//! - `remove`はtrait objectを`Box<dyn Any + Send + Sync>`へupcastしてからdowncastする
//! - `statuses`はdyn methodを呼び、`HashMap`の反復順に依存しない順で返す
//!
//! `Any`の判定対象はexact concrete typeであり、「あるtraitを実装する型か」は判定できません
//! また`Any: 'static`なので、登録値は非`'static`なborrowを内包できません
//! downcastは通常処理ではなく、型固有機能が必要な境界で使うescape hatchです
//!
//! Rust 1.86で安定化されたtrait upcastingを使い、旧式の`as_any` helperを
//! `Extension`へ追加しないでください

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ExtensionStatus {
    name: String,
    detail: String,
}

impl ExtensionStatus {
    fn new(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            detail: detail.into(),
        }
    }
}

trait Extension: Any + Send + Sync {
    fn status(&self) -> ExtensionStatus;
}

#[derive(Default)]
struct ExtensionRegistry {
    extensions: HashMap<TypeId, Box<dyn Extension + Send + Sync>>,
}

impl ExtensionRegistry {
    fn len(&self) -> usize {
        self.extensions.len()
    }

    fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }

    /// 同じexact concrete typeがあれば、以前の所有値を返す
    fn insert<E>(&mut self, extension: E) -> Option<Box<E>>
    where
        E: Extension,
    {
        todo!(
            "{}のTypeIdで挿入し、以前の値をowned downcastしてください: 現在{}件、status={:?}",
            std::any::type_name::<E>(),
            self.extensions.len(),
            extension.status()
        )
    }

    fn get<E>(&self) -> Option<&E>
    where
        E: Extension,
    {
        todo!(
            "{}のTypeIdで検索し、&dyn Anyへupcastしてdowncast_refしてください: 現在{}件",
            std::any::type_name::<E>(),
            self.extensions.len()
        )
    }

    fn get_mut<E>(&mut self) -> Option<&mut E>
    where
        E: Extension,
    {
        todo!(
            "{}のTypeIdで検索し、&mut dyn Anyへupcastしてdowncast_mutしてください: 現在{}件",
            std::any::type_name::<E>(),
            self.extensions.len()
        )
    }

    /// registryから値を外し、元の具体型を所有する`Box`へ戻す
    fn remove<E>(&mut self) -> Option<Box<E>>
    where
        E: Extension,
    {
        todo!(
            "{}の値をremoveし、Box<dyn Any + Send + Sync>経由でowned downcastしてください: 現在{}件",
            std::any::type_name::<E>(),
            self.extensions.len()
        )
    }

    /// `HashMap`の反復順へ依存しないようstatusの内容でsortする
    fn statuses(&self) -> Vec<ExtensionStatus> {
        todo!(
            "{}個のtrait objectへstatusをdyn dispatchし、結果をsortしてください",
            self.extensions.len()
        )
    }
}

#[derive(Debug)]
struct CounterExtension {
    name: String,
    count: u64,
}

impl CounterExtension {
    fn new(name: &str, count: u64) -> Self {
        Self {
            name: name.to_string(),
            count,
        }
    }

    fn increment(&mut self) {
        self.count = self.count.saturating_add(1);
    }
}

impl Extension for CounterExtension {
    fn status(&self) -> ExtensionStatus {
        ExtensionStatus::new(&self.name, format!("{}件処理済み", self.count))
    }
}

#[derive(Debug)]
struct NoticeExtension {
    name: String,
    message: String,
}

impl NoticeExtension {
    fn new(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            message: message.to_string(),
        }
    }
}

impl Extension for NoticeExtension {
    fn status(&self) -> ExtensionStatus {
        ExtensionStatus::new(&self.name, &self.message)
    }
}

fn main() {
    let mut registry = ExtensionRegistry::default();
    registry.insert(CounterExtension::new("配送件数", 12));
    registry.insert(NoticeExtension::new("運行情報", "正常運転中🤖"));

    for status in registry.statuses() {
        println!("{}: {}", status.name, status.detail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn 空のregistryでは全ての検索と削除がnoneになる() {
        let mut registry = ExtensionRegistry::default();

        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert!(registry.get::<CounterExtension>().is_none());
        assert!(registry.get_mut::<CounterExtension>().is_none());
        assert!(registry.remove::<CounterExtension>().is_none());
        assert!(registry.statuses().is_empty());
    }

    struct WrappedCounter(CounterExtension);

    impl Extension for WrappedCounter {
        fn status(&self) -> ExtensionStatus {
            ExtensionStatus::new("wrapped", format!("内側={}", self.0.count))
        }
    }

    #[test]
    fn exact_typeが異なるextensionは同時に保持できる() {
        let mut registry = ExtensionRegistry::default();
        registry.insert(CounterExtension::new("counter", 4));
        registry.insert(WrappedCounter(CounterExtension::new("inner", 9)));
        registry.insert(NoticeExtension::new("notice", "ready"));

        assert_eq!(registry.len(), 3);
        assert_eq!(registry.get::<CounterExtension>().unwrap().count, 4);
        assert_eq!(registry.get::<WrappedCounter>().unwrap().0.count, 9);
        assert_eq!(registry.get::<NoticeExtension>().unwrap().message, "ready");
    }

    #[test]
    fn 同じtypeのinsertは以前の所有値を返す() {
        let mut registry = ExtensionRegistry::default();
        let previous = CounterExtension::new("old", 1);
        let previous_name_pointer = previous.name.as_ptr();
        assert!(registry.insert(previous).is_none());

        let returned = registry
            .insert(CounterExtension::new("new", 2))
            .expect("同型の値を置換する");

        assert_eq!(registry.len(), 1);
        assert_eq!(returned.name, "old");
        assert_eq!(returned.name.as_ptr(), previous_name_pointer);
        assert_eq!(registry.get::<CounterExtension>().unwrap().name, "new");
    }

    #[test]
    fn get_mutで型固有の状態を変更しdynのstatusへ反映できる() {
        let mut registry = ExtensionRegistry::default();
        registry.insert(CounterExtension::new("処理数", 40));

        let counter = registry.get_mut::<CounterExtension>().unwrap();
        counter.increment();
        counter.increment();

        assert_eq!(registry.get::<CounterExtension>().unwrap().count, 42);
        assert_eq!(
            registry.statuses(),
            [ExtensionStatus::new("処理数", "42件処理済み")]
        );
    }

    #[test]
    fn removeは具体型とallocationの所有権を返して他の型を残す() {
        let mut registry = ExtensionRegistry::default();
        let notice = NoticeExtension::new("notice", "返却対象");
        let message_pointer = notice.message.as_ptr();
        registry.insert(notice);
        registry.insert(CounterExtension::new("counter", 3));

        let removed = registry
            .remove::<NoticeExtension>()
            .expect("登録値を取り出せる");

        assert_eq!(removed.message, "返却対象");
        assert_eq!(removed.message.as_ptr(), message_pointer);
        assert!(registry.get::<NoticeExtension>().is_none());
        assert!(registry.get::<CounterExtension>().is_some());
        assert_eq!(registry.len(), 1);
    }

    fn status_registry(reverse: bool) -> ExtensionRegistry {
        let mut registry = ExtensionRegistry::default();
        if reverse {
            registry.insert(CounterExtension::new("z-counter", 7));
            registry.insert(NoticeExtension::new("a-notice", "ready"));
        } else {
            registry.insert(NoticeExtension::new("a-notice", "ready"));
            registry.insert(CounterExtension::new("z-counter", 7));
        }
        registry
    }

    #[test]
    fn statusはhashmapの登録順に依存せず決定的に並ぶ() {
        let forward = status_registry(false).statuses();
        let reverse = status_registry(true).statuses();

        assert_eq!(forward, reverse);
        assert_eq!(
            forward,
            [
                ExtensionStatus::new("a-notice", "ready"),
                ExtensionStatus::new("z-counter", "7件処理済み"),
            ]
        );
    }

    #[test]
    fn utf8のnameとdetailをstatusで保持する() {
        let mut registry = ExtensionRegistry::default();
        registry.insert(NoticeExtension::new("東京大学🤖", "温度=24℃・状態=正常"));

        assert_eq!(
            registry.statuses(),
            [ExtensionStatus::new("東京大学🤖", "温度=24℃・状態=正常")]
        );
    }

    #[test]
    fn registryはsendかつsyncである() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<ExtensionRegistry>();
    }

    struct DropExtension {
        name: String,
        drops: Arc<AtomicUsize>,
    }

    impl Extension for DropExtension {
        fn status(&self) -> ExtensionStatus {
            ExtensionStatus::new(&self.name, "drop確認")
        }
    }

    impl Drop for DropExtension {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn 置換とremoveとregistry_dropで各値を一度だけdropする() {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut registry = ExtensionRegistry::default();
        registry.insert(DropExtension {
            name: String::from("old"),
            drops: Arc::clone(&drops),
        });
        let old = registry
            .insert(DropExtension {
                name: String::from("new"),
                drops: Arc::clone(&drops),
            })
            .expect("oldを所有値として返す");

        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(old);
        assert_eq!(drops.load(Ordering::SeqCst), 1);

        let new = registry.remove::<DropExtension>().unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        drop(new);
        assert_eq!(drops.load(Ordering::SeqCst), 2);

        registry.insert(DropExtension {
            name: String::from("registry-owned"),
            drops: Arc::clone(&drops),
        });
        drop(registry);
        assert_eq!(drops.load(Ordering::SeqCst), 3);
    }
}

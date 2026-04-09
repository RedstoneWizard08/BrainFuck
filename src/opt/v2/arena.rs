use once_cell::sync::Lazy;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::{
    collections::HashSet,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, Mutex},
};

type ArcArena = Arc<dyn AnyArena>;
static ARENA_POOL: Lazy<Arc<Mutex<Vec<ArcArena>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArenaItem<T: Default + Send + Sync + 'static> {
    index: usize,
    arena: usize,

    _ty: PhantomData<T>,
}

impl<T: Default + Send + Sync + 'static> ArenaItem<T> {
    pub fn of(arena: &ArenaRef<T>, index: usize) -> ArenaItem<T> {
        ArenaItem {
            index: index,
            arena: arena.idx,

            _ty: PhantomData,
        }
    }
}

impl<T: Clone + Default + Send + Sync + 'static> Copy for ArenaItem<T> {}

pub trait AnyArena: Send + Sync {
    fn drop(&self, item: usize);
}

pub trait BaseArena {
    type Item: Default + Send + Sync + 'static;

    fn state(&self) -> &Arc<RwLock<ArenaState<Self::Item>>>;
}

pub fn finalize_arena<T: Default + Send + Sync + 'static>(arena: Arena<T>) -> ArenaRef<T> {
    let mut guard = ARENA_POOL.lock().unwrap();
    let idx = guard.len();
    let arc = Arc::new(arena);

    guard.push(Arc::clone(&arc) as ArcArena);

    ArenaRef::new(arc, idx)
}

#[derive(Debug, Clone)]
pub struct ArenaRef<T: Default + Send + Sync + 'static> {
    arc: Arc<Arena<T>>,
    idx: usize,
}

impl<T: Default + Send + Sync + 'static> ArenaRef<T> {
    pub fn new(arc: Arc<Arena<T>>, idx: usize) -> ArenaRef<T> {
        Self { arc, idx }
    }

    pub fn alloc(&self) -> ArenaItem<T> {
        let item = T::default();
        let mut guard = self.arc.state().write();

        if guard.vacancies.is_empty() {
            let idx = guard.arena.len();

            guard.arena.push(item);

            ArenaItem::of(self, idx)
        } else {
            let idx = *guard.vacancies.iter().next().unwrap();

            guard.vacancies.remove(&idx);
            guard.arena[idx] = item;

            ArenaItem::of(self, idx)
        }
    }

    pub fn fetch(&self, item: ArenaItem<T>) -> MappedRwLockReadGuard<'_, T> {
        let guard = self.state.read_recursive();

        RwLockReadGuard::map(guard, |it| &it.arena[item.index])
    }

    pub fn fetch_mut(&self, item: ArenaItem<T>) -> MappedRwLockWriteGuard<'_, T> {
        let guard = self.state.write();

        RwLockWriteGuard::map(guard, |it| &mut it.arena[item.index])
    }

    pub fn all(&self) -> MappedRwLockReadGuard<'_, Vec<T>> {
        let guard = self.state.read_recursive();

        RwLockReadGuard::map(guard, |it| &it.arena)
    }

    pub fn all_mut(&self) -> MappedRwLockWriteGuard<'_, Vec<T>> {
        let guard = self.state.write();

        RwLockWriteGuard::map(guard, |it| &mut it.arena)
    }
}

impl<T: Default + Send + Sync + 'static> Deref for ArenaRef<T> {
    type Target = Arena<T>;

    fn deref(&self) -> &Self::Target {
        &self.arc
    }
}

#[derive(Debug)]
pub struct ArenaState<T: Default + Send + Sync + 'static> {
    arena: Vec<T>,
    vacancies: HashSet<usize>,
}

impl<T: Default + Send + Sync + 'static> ArenaState<T> {
    pub fn new() -> Self {
        Self {
            arena: Vec::new(),
            vacancies: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct Arena<T: Default + Send + Sync + 'static> {
    state: Arc<RwLock<ArenaState<T>>>,
}

impl<T: Default + Send + Sync + 'static> Arena<T> {
    pub fn new() -> ArenaRef<T> {
        finalize_arena(Self {
            state: Arc::new(RwLock::new(ArenaState::new())),
        })
    }

    pub fn drop_item(&self, item: ArenaItem<T>) {
        self.drop_index(item.index);
    }

    pub fn drop_index(&self, item: usize) {
        if self.state().is_locked() {
            panic!("State was locked!");
        }

        let mut guard = self.state().write();

        if guard.vacancies.contains(&item) {
            return;
        }

        guard.arena[item] = T::default();
        guard.vacancies.insert(item);
    }
}

impl<T: Default + Send + Sync + 'static> AnyArena for Arena<T> {
    fn drop(&self, item: usize) {
        self.drop_index(item);
    }
}

impl<T: Default + Send + Sync + 'static> BaseArena for Arena<T> {
    type Item = T;

    fn state(&self) -> &Arc<RwLock<ArenaState<Self::Item>>> {
        &self.state
    }
}

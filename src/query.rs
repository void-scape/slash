use bevy::{
    ecs::{
        query::{QueryData, QueryEntityError, QueryFilter, ROQueryItem},
        relationship::Relationship,
        system::SystemParam,
    },
    prelude::*,
};

#[derive(SystemParam)]
pub struct AncestorQuery<
    'w,
    's,
    D: QueryData + 'static,
    F: QueryFilter + 'static = (),
    R: Relationship = ChildOf,
> {
    ancestors: Query<'w, 's, &'static R>,
    query: Query<'w, 's, D, F>,
}

#[derive(Debug, Clone)]
pub enum AncestorQueryError {
    NoMatchingEntity,
    Query(QueryEntityError),
}

impl core::fmt::Display for AncestorQueryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoMatchingEntity => {
                write!(f, "no matching entity for ancestor query")
            }
            Self::Query(q) => q.fmt(f),
        }
    }
}

impl core::error::Error for AncestorQueryError {}

#[allow(unused)]
impl<'w, 's, D, F, R> AncestorQuery<'w, 's, D, F, R>
where
    D: QueryData + 'static,
    F: QueryFilter + 'static,
    R: Relationship,
{
    pub fn get(&self, target: Entity) -> Result<ROQueryItem<'_, 's, D>, AncestorQueryError> {
        for ancestor in self.ancestors.iter_ancestors(target) {
            if let Ok(data) = self.query.get(ancestor) {
                return Ok(data);
            }
        }

        Err(AncestorQueryError::NoMatchingEntity)
    }

    pub fn get_mut(&mut self, target: Entity) -> Result<D::Item<'_, 's>, AncestorQueryError> {
        let mut target_ancestor = None;
        for ancestor in self.ancestors.iter_ancestors(target) {
            if self.query.contains(ancestor) {
                target_ancestor = Some(ancestor);
                break;
            }
        }

        match target_ancestor {
            Some(ancestor) => self
                .query
                .get_mut(ancestor)
                .map_err(AncestorQueryError::Query),
            None => Err(AncestorQueryError::NoMatchingEntity),
        }
    }

    pub fn get_last(&self, target: Entity) -> Result<ROQueryItem<'_, 's, D>, AncestorQueryError> {
        let mut target_ancestor = None;
        for ancestor in self.ancestors.iter_ancestors(target) {
            if self.query.contains(ancestor) {
                target_ancestor = Some(ancestor);
            }
        }

        match target_ancestor {
            Some(ancestor) => self.query.get(ancestor).map_err(AncestorQueryError::Query),
            None => Err(AncestorQueryError::NoMatchingEntity),
        }
    }

    pub fn get_last_mut(&mut self, target: Entity) -> Result<D::Item<'_, 's>, AncestorQueryError> {
        let mut target_ancestor = None;
        for ancestor in self.ancestors.iter_ancestors(target) {
            if self.query.contains(ancestor) {
                target_ancestor = Some(ancestor);
            }
        }

        match target_ancestor {
            Some(ancestor) => self
                .query
                .get_mut(ancestor)
                .map_err(AncestorQueryError::Query),
            None => Err(AncestorQueryError::NoMatchingEntity),
        }
    }

    pub fn get_inclusive(
        &self,
        target: Entity,
    ) -> Result<ROQueryItem<'_, 's, D>, AncestorQueryError> {
        for ancestor in std::iter::once(target).chain(self.ancestors.iter_ancestors(target)) {
            if let Ok(data) = self.query.get(ancestor) {
                return Ok(data);
            }
        }

        Err(AncestorQueryError::NoMatchingEntity)
    }

    pub fn get_inclusive_mut(
        &mut self,
        target: Entity,
    ) -> Result<D::Item<'_, 's>, AncestorQueryError> {
        let mut target_ancestor = None;
        for ancestor in std::iter::once(target).chain(self.ancestors.iter_ancestors(target)) {
            if self.query.contains(ancestor) {
                target_ancestor = Some(ancestor);
                break;
            }
        }

        match target_ancestor {
            Some(ancestor) => self
                .query
                .get_mut(ancestor)
                .map_err(AncestorQueryError::Query),
            None => Err(AncestorQueryError::NoMatchingEntity),
        }
    }
}

use crate::raw_event::RawSubject;
use crate::{SubjectData, SubjectError, SubjectId};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::{Index, IndexMut};

#[derive(Default)]
pub struct SubjectMap<T = ()>(BTreeMap<SubjectId, (SubjectData, T)>);

impl<T> Index<SubjectId> for SubjectMap<T> {
    type Output = (SubjectData, T);

    fn index(&self, index: SubjectId) -> &Self::Output {
        self.0
            .get(&index)
            .expect("subject id created without matching subject data")
    }
}

impl<T> IndexMut<SubjectId> for SubjectMap<T> {
    fn index_mut(&mut self, index: SubjectId) -> &mut Self::Output {
        self.0
            .get_mut(&index)
            .expect("subject id created without matching subject data")
    }
}

impl<T: Default> SubjectMap<T> {
    pub fn insert(&mut self, raw: &RawSubject) -> Result<(SubjectId, &mut T), SubjectError> {
        let id = raw.id()?;
        let (_, data) = self
            .0
            .entry(id)
            .or_insert_with(|| (raw.try_into().unwrap(), T::default()));
        Ok((id, data))
    }
}

impl<T> SubjectMap<T> {
    pub fn to_just_subjects(&self) -> SubjectMap<()> {
        SubjectMap(
            self.0
                .iter()
                .map(|(k, (a, _))| (*k, (a.clone(), ())))
                .collect(),
        )
    }
}

impl<T> IntoIterator for SubjectMap<T> {
    type Item = (SubjectId, SubjectData, T);
    type IntoIter = SubjectMapIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        SubjectMapIter {
            iter: self.0.into_iter(),
        }
    }
}

pub struct SubjectMapIter<T> {
    iter: std::collections::btree_map::IntoIter<SubjectId, (SubjectData, T)>,
}

impl<T> Iterator for SubjectMapIter<T> {
    type Item = (SubjectId, SubjectData, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, (a, b))| (k, a, b))
    }
}

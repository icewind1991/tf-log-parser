use crate::common::SubjectId;
use crate::event::{EventMeta, GameEvent};
use crate::raw_event::RawEventType;
use crate::{SubjectData, SubjectMap};
pub use chat::{ChatMessage, ChatMessages, ChatType};
pub use classstats::{ClassStats, ClassStatsHandler};
pub use healspread::HealSpread;
pub use lobbysettings::{
    LobbySettingsError, LobbySettingsHandler, Location, Settings as LobbySettings,
};
pub use medicstats::{MedicStats, MedicStatsBuilder};
use serde::Serialize;
use std::marker::PhantomData;

mod chat;
mod classstats;
mod healspread;
mod lobbysettings;
mod medicstats;

pub trait EventHandler: Default {
    type GlobalOutput;
    type PerSubjectData: Default;
    type PerSubjectOutput;

    fn does_handle(ty: RawEventType) -> bool;

    fn handle(
        &mut self,
        meta: &EventMeta,
        subject: SubjectId,
        subject_data: &mut Self::PerSubjectData,
        event: &GameEvent,
    );

    fn finish_global(self, subjects: &SubjectMap) -> Self::GlobalOutput;

    fn finish_per_subject(
        &mut self,
        subject: &SubjectData,
        data: Self::PerSubjectData,
    ) -> Self::PerSubjectOutput;
}

#[derive(Default)]
pub struct HandlerStack<Head, Tail> {
    head: Head,
    tail: Tail,
}

impl<Head: EventHandler, Tail: EventHandler> EventHandler for HandlerStack<Head, Tail> {
    type GlobalOutput = (Head::GlobalOutput, Tail::GlobalOutput);
    type PerSubjectData = (Head::PerSubjectData, Tail::PerSubjectData);
    type PerSubjectOutput = (Head::PerSubjectOutput, Tail::PerSubjectOutput);

    fn does_handle(ty: RawEventType) -> bool {
        Head::does_handle(ty) || Tail::does_handle(ty)
    }

    fn handle(
        &mut self,
        meta: &EventMeta,
        subject: SubjectId,
        subject_data: &mut Self::PerSubjectData,
        event: &GameEvent,
    ) {
        self.head.handle(meta, subject, &mut subject_data.0, event);
        self.tail.handle(meta, subject, &mut subject_data.1, event);
    }

    fn finish_global(self, subjects: &SubjectMap) -> Self::GlobalOutput {
        (
            self.head.finish_global(subjects),
            self.tail.finish_global(subjects),
        )
    }

    fn finish_per_subject(
        &mut self,
        subject: &SubjectData,
        data: Self::PerSubjectData,
    ) -> Self::PerSubjectOutput {
        (
            self.head.finish_per_subject(subject, data.0),
            self.tail.finish_per_subject(subject, data.1),
        )
    }
}

#[macro_export]
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

#[macro_export]
macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ $crate::replace_expr!($tts 1usize))*};
}

#[macro_export]
macro_rules! handler {
    ($name:ident {$($child:ident: $ty:path),*}) => {
        handler!($name { $($child: $ty,)* } );
    };
    ($name:ident {$($child:ident: $ty:path,)*}) => {
        paste::paste! {
            #[derive(Default)]
            pub struct $name {
                pub $($child: $ty),*
            }

            pub struct [<$name GlobalOutput>] {
                pub $($child: <$ty as $crate::EventHandler>::GlobalOutput),*
            }

            impl serde::Serialize for [<$name GlobalOutput>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeStruct;
                    let mut state = serializer.serialize_struct(concat!(stringify!($name), "output"), $crate::count_tts!($($child)*))?;
                    $(
                        if self.$child != <<$ty as $crate::EventHandler>::GlobalOutput>::default() {
                            state.serialize_field(stringify!($child), &self.$child)?;
                        }
                    )*
                    state.end()
                }
            }

            pub struct [<$name PerSubjectData>] {
                $($child: <$ty as $crate::EventHandler>::PerSubjectData),*
            }

            impl Default for [<$name PerSubjectData>] {
                fn default() -> Self {
                    Self {
                        $($child: <$ty as $crate::EventHandler>::PerSubjectData::default()),*
                    }
                }
            }

            pub struct [<$name PerSubjectOutput>] {
                pub $($child: <$ty as $crate::EventHandler>::PerSubjectOutput),*
            }

            impl serde::Serialize for [<$name PerSubjectOutput>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeStruct;
                    let mut state = serializer.serialize_struct(concat!(stringify!($name), "output"), $crate::count_tts!($($child)*))?;
                    $(
                        if self.$child != <<$ty as $crate::EventHandler>::PerSubjectOutput>::default() {
                            state.serialize_field(stringify!($child), &self.$child)?;
                        }
                    )*
                    state.end()
                }
            }


            impl $crate::EventHandler for $name {
                type GlobalOutput = [<$name GlobalOutput>];
                type PerSubjectData = [<$name PerSubjectData>];
                type PerSubjectOutput = [<$name PerSubjectOutput>];

                fn does_handle(ty: $crate::RawEventType) -> bool {
                    #[allow(unused_imports)]
                    use $crate::EventHandler;
                    $($ty::does_handle(ty))||*
                }

                fn handle(&mut self, meta: &$crate::EventMeta, subject: $crate::SubjectId,subject_data:&mut  Self::PerSubjectData, event: &$crate::GameEvent) {
                    #[allow(unused_imports)]
                    use $crate::EventHandler;
                    $(self.$child.handle(meta, subject, &mut subject_data.$child, event);)*
                }

                fn finish_global(self, subjects: &$crate::SubjectMap) -> Self::GlobalOutput {
                    #[allow(unused_imports)]
                    use $crate::EventHandler;
                    Self::GlobalOutput {
                        $($child: self.$child.finish_global(subjects),)*
                    }
                }

                fn finish_per_subject(&mut self, subject: &$crate::SubjectData, data: Self::PerSubjectData) -> Self::PerSubjectOutput {
                    Self::PerSubjectOutput {
                        $($child: self.$child.finish_per_subject(subject, data.$child),)*
                    }
                }
            }
        }
    };
}

pub trait GlobalData: Default {
    type Output;

    fn does_handle(ty: RawEventType) -> bool;
    fn handle_event(&mut self, meta: &EventMeta, subject: SubjectId, event: &GameEvent);
    fn finish(self, subjects: &SubjectMap) -> Self::Output;
}

impl<T: GlobalData> EventHandler for T {
    type GlobalOutput = T::Output;
    type PerSubjectData = ();
    type PerSubjectOutput = ();

    fn does_handle(ty: RawEventType) -> bool {
        T::does_handle(ty)
    }

    fn handle(
        &mut self,
        meta: &EventMeta,
        subject: SubjectId,
        _subject_data: &mut Self::PerSubjectData,
        event: &GameEvent,
    ) {
        self.handle_event(meta, subject, event)
    }

    fn finish_global(self, subjects: &SubjectMap) -> Self::GlobalOutput {
        self.finish(subjects)
    }

    fn finish_per_subject(
        &mut self,
        _subject: &SubjectData,
        _data: Self::PerSubjectData,
    ) -> Self::PerSubjectOutput {
    }
}

pub trait PlayerSpecificData: Default {
    type Output: Serialize;

    fn does_handle(ty: RawEventType) -> bool;
    fn handle_event(&mut self, meta: &EventMeta, subject: SubjectId, event: &GameEvent);
    fn finish(self) -> Self::Output;
}

#[derive(Default)]
pub struct PlayerHandler<T: PlayerSpecificData>(PhantomData<T>);

impl<T: PlayerSpecificData + Default> EventHandler for PlayerHandler<T> {
    type GlobalOutput = ();
    type PerSubjectData = T;
    type PerSubjectOutput = T::Output;

    fn does_handle(ty: RawEventType) -> bool {
        T::does_handle(ty)
    }

    fn handle(
        &mut self,
        meta: &EventMeta,
        subject: SubjectId,
        subject_data: &mut Self::PerSubjectData,
        event: &GameEvent,
    ) {
        subject_data.handle_event(meta, subject, event)
    }

    fn finish_global(self, _subjects: &SubjectMap) -> Self::GlobalOutput {}

    fn finish_per_subject(
        &mut self,
        _subject: &SubjectData,
        data: Self::PerSubjectData,
    ) -> Self::PerSubjectOutput {
        data.finish()
    }
}

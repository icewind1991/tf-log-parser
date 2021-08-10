use crate::common::SubjectId;
use crate::event::GameEvent;
use crate::raw_event::RawEventType;
use crate::{SubjectData, SubjectMap};
pub use chat::{ChatHandler, ChatMessage, ChatType};
pub use classstats::{ClassStat, ClassStatsHandler};
pub use healspread::HealSpreadHandler;
pub use lobbysettings::{
    LobbySettingsError, LobbySettingsHandler, Location, Settings as LobbySettings,
};
pub use medicstats::{MedicStats, MedicStatsHandler};

mod chat;
mod classstats;
mod healspread;
mod lobbysettings;
mod medicstats;

pub trait EventHandler: Default {
    type GlobalOutput;
    type PerSubjectData: Default;
    type PerSubjectOutput;

    fn does_handle(&self, ty: RawEventType) -> bool;

    fn handle(
        &mut self,
        time: u32,
        subject: SubjectId,
        subject_data: &mut Self::PerSubjectData,
        event: &GameEvent,
    );

    fn finish_global(self, subjects: &SubjectMap) -> Self::GlobalOutput;

    fn finish_per_subject(
        &self,
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

    fn does_handle(&self, ty: RawEventType) -> bool {
        self.head.does_handle(ty) || self.tail.does_handle(ty)
    }

    fn handle(
        &mut self,
        time: u32,
        subject: SubjectId,
        subject_data: &mut Self::PerSubjectData,
        event: &GameEvent,
    ) {
        self.head.handle(time, subject, &mut subject_data.0, event);
        self.tail.handle(time, subject, &mut subject_data.1, event);
    }

    fn finish_global(self, subjects: &SubjectMap) -> Self::GlobalOutput {
        (
            self.head.finish_global(subjects),
            self.tail.finish_global(subjects),
        )
    }

    fn finish_per_subject(
        &self,
        subject: &SubjectData,
        data: Self::PerSubjectData,
    ) -> Self::PerSubjectOutput {
        (
            self.head.finish_per_subject(subject, data.0),
            self.tail.finish_per_subject(subject, data.1),
        )
    }
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
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
                $($child: $ty),*
            }

            pub struct [<$name GlobalOutput>] {
                $($child: <$ty as $crate::EventHandler>::GlobalOutput),*
            }

            impl serde::Serialize for [<$name GlobalOutput>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeStruct;
                    let mut state = serializer.serialize_struct(concat!(stringify!($name), "output"), count_tts!($($child)*))?;
                    $(state.serialize_field(stringify!($child), &self.$child)?;)*
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
                $($child: <$ty as $crate::EventHandler>::PerSubjectOutput),*
            }

            impl serde::Serialize for [<$name PerSubjectOutput>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeStruct;
                    let mut state = serializer.serialize_struct(concat!(stringify!($name), "output"), count_tts!($($child)*))?;
                    $(state.serialize_field(stringify!($child), &self.$child)?;)*
                    state.end()
                }
            }


            impl $crate::EventHandler for $name {
                type GlobalOutput = [<$name GlobalOutput>];
                type PerSubjectData = [<$name PerSubjectData>];
                type PerSubjectOutput = [<$name PerSubjectOutput>];

                fn does_handle(&self, ty: $crate::RawEventType) -> bool {
                    #[allow(unused_imports)]
                    use $crate::EventHandler;
                    $(self.$child.does_handle(ty))||*
                }

                fn handle(&mut self, time: u32, subject: $crate::SubjectId,subject_data:&mut  Self::PerSubjectData, event: &$crate::GameEvent) {
                    #[allow(unused_imports)]
                    use $crate::EventHandler;
                    $(self.$child.handle(time, subject, &mut subject_data.$child, event);)*
                }

                fn finish_global(self, subjects: &$crate::SubjectMap) -> Self::GlobalOutput {
                    #[allow(unused_imports)]
                    use $crate::EventHandler;
                    Self::GlobalOutput {
                        $($child: self.$child.finish_global(subjects),)*
                    }
                }

                fn finish_per_subject(&self, subject: &SubjectData, data: Self::PerSubjectData) -> Self::PerSubjectOutput {
                    Self::PerSubjectOutput {
                        $($child: self.$child.finish_per_subject(subject, data.$child),)*
                    }
                }
            }
        }
    };
}

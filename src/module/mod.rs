use crate::common::SubjectId;
use crate::event::GameEvent;
use crate::raw_event::RawEventType;
use crate::SubjectMap;
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
    type Output;

    fn does_handle(&self, ty: RawEventType) -> bool;

    fn handle(&mut self, time: u32, subject: SubjectId, event: &GameEvent);

    fn finish(self, subjects: &SubjectMap) -> Self::Output;
}

#[derive(Default)]
pub struct HandlerStack<Head, Tail> {
    head: Head,
    tail: Tail,
}

impl<Head: EventHandler, Tail: EventHandler> EventHandler for HandlerStack<Head, Tail> {
    type Output = (Head::Output, Tail::Output);

    fn does_handle(&self, ty: RawEventType) -> bool {
        self.head.does_handle(ty) || self.tail.does_handle(ty)
    }

    fn handle(&mut self, time: u32, subject: SubjectId, event: &GameEvent) {
        self.head.handle(time, subject, event);
        self.tail.handle(time, subject, event);
    }

    fn finish(self, subjects: &SubjectMap) -> Self::Output {
        (self.head.finish(subjects), self.tail.finish(subjects))
    }
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

            pub struct [<$name Output>] {
                $($child: <$ty as $crate::EventHandler>::Output),*
            }

            impl serde::Serialize for [<$name Output>] {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeStruct;
                    let mut state = serializer.serialize_struct(concat!("$name", "output"), 3)?;
                    $(state.serialize_field("$child", &self.$child)?;)*
                    state.end()
                }
            }


            impl $crate::EventHandler for $name {
                type Output = [<$name Output>];

                fn does_handle(&self, ty: $crate::RawEventType) -> bool {
                    #[allow(unused_imports)]
                    use $crate::EventHandler;
                    $(self.$child.does_handle(ty))||*
                }

                fn handle(&mut self, time: u32, subject: $crate::SubjectId, event: &$crate::GameEvent) {
                    #[allow(unused_imports)]
                    use $crate::EventHandler;
                    $(self.$child.handle(time, subject, event);)*
                }

                fn finish(self, subjects: &$crate::SubjectMap) -> Self::Output {
                    #[allow(unused_imports)]
                    use $crate::EventHandler;
                    Self::Output {
                        $($child: self.$child.finish(subjects),)*
                    }
                }
            }
        }
    };
}

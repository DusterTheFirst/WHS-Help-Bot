use crate::bot_data::{SqliteDatabaseConnection};
use crate::db::models::DatabaseClass;
use crate::db::schema::classes as database_classes;
use diesel::prelude::*;
use serenity::{
    framework::standard::{
        macros::{command},
        Args, CommandResult,
    },
    model::{channel::Message},
    prelude::*,
    utils::Colour,
};
use std::collections::BTreeMap;

#[command]
#[description = "List the classes that you have."]
#[usage = "[filter]"]
#[only_in(guilds)]
pub fn mine(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read();

    let db: &SqliteConnection = &data
        .get::<SqliteDatabaseConnection>()
        .unwrap()
        .lock()
        .unwrap();

    let filter = args.remains();

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            match database_classes::table
                .filter(
                    database_classes::id.eq_any(
                        msg.member
                            .as_ref()
                            .map(|x| x.roles.iter().map(|x| x.to_string()).collect::<Vec<_>>())
                            .unwrap_or_else(|| vec![]),
                    ),
                )
                .load::<DatabaseClass>(db)
            {
                Err(err) => {
                    e.title("Error Loading Classes");
                    e.color(Colour::DARK_RED);
                    e.description(format!("```{:#?}```", err));
                }
                Ok(classes) => {
                    let classes = match filter {
                        Some(f) => {
                            let classes_len = classes.len();
                            let newclasses = classes
                                .into_iter()
                                .filter(|x| {
                                    (x.tag.to_lowercase().contains(&f.to_lowercase())
                                        || x.parse_role_id()
                                            .to_role_cached(&ctx)
                                            .map(|r| {
                                                r.name.to_lowercase().contains(&f.to_lowercase())
                                            })
                                            .unwrap_or(false))
                                })
                                .collect::<Vec<_>>();

                            e.title(format!(
                                "Displaying {} of {}",
                                newclasses.len(),
                                classes_len,
                            ));
                            e.footer(|footer| {
                                footer.text(format!(
                                    "Filter: `{}`, User: `{}`",
                                    f,
                                    msg.author.tag()
                                ))
                            });

                            newclasses
                        }
                        None => {
                            e.title(format!("Displaying {} classes", classes.len()));
                            e.footer(|footer| footer.text(format!("User: `{}`", msg.author.tag())));

                            classes
                        }
                    };

                    if classes.is_empty() {
                        e.color(Colour::DARK_GOLD);
                    } else {
                        e.color(Colour::DARK_GREEN);
                    }

                    let mut tags = BTreeMap::<String, Vec<DatabaseClass>>::new();

                    for class in classes {
                        tags.entry(class.tag.clone())
                            .or_insert_with(Vec::new)
                            .push(class);
                    }

                    for (tag_name, tag_classes) in tags.iter() {
                        e.field(
                            match tag_name.len() {
                                0 => "None",
                                _ => &tag_name,
                            },
                            tag_classes
                                .iter()
                                .map(|class| {
                                    format!(
                                        "**{}**\nRole: {}\nId: `{}`",
                                        class
                                            .parse_role_id()
                                            .to_role_cached(&ctx)
                                            .map(|x| x.name)
                                            .unwrap_or_else(|| "NONEXISTANT".to_owned()),
                                        class.parse_role_id().mention(),
                                        class.id
                                    )
                                })
                                .collect::<Vec<String>>()
                                .join("\n"),
                            true,
                        );
                    }
                }
            };

            e
        });

        m
    })?;

    Ok(())
}
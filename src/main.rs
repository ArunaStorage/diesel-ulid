use aruna_ulid::ArunaUlid;
mod diesel_test;
mod schema;

use diesel::prelude::*;
use schema::posts;

use crate::diesel_test::establish_connection;

#[derive(Queryable, Insertable, Identifiable, Debug)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: ArunaUlid,
    pub body: String,
    pub published: bool,
}


fn main() {
    use self::schema::posts::dsl::*;

    let connection = &mut establish_connection();


    let new_post = Post {id: ArunaUlid::generate(), body: "A body".to_string(), published: false };

    let res = diesel::insert_into(posts)
        .values(&new_post)
        .get_result::<Post>(connection)
        .expect("Error saving new post");

    println!("Inserted: {:?}", res);

    let results = posts
        .limit(5)
        .load::<Post>(connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{:?}", post);
    }
}

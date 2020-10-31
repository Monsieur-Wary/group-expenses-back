table! {
    expenses (id) {
        id -> Uuid,
        group_id -> Uuid,
        person_id -> Uuid,
        name -> Varchar,
        amount -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    groups (id) {
        id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    persons (id) {
        id -> Uuid,
        group_id -> Uuid,
        name -> Varchar,
        resources -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        password -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(expenses -> groups (group_id));
joinable!(expenses -> persons (person_id));
joinable!(groups -> users (user_id));
joinable!(persons -> groups (group_id));

allow_tables_to_appear_in_same_query!(expenses, groups, persons, users,);

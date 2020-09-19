table! {
    dashboards (id) {
        id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    expenses (id) {
        id -> Uuid,
        dashboard_id -> Uuid,
        person_id -> Uuid,
        name -> Varchar,
        amount -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    persons (id) {
        id -> Uuid,
        dashboard_id -> Uuid,
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

joinable!(dashboards -> users (user_id));
joinable!(expenses -> dashboards (dashboard_id));
joinable!(expenses -> persons (person_id));
joinable!(persons -> dashboards (dashboard_id));

allow_tables_to_appear_in_same_query!(dashboards, expenses, persons, users,);

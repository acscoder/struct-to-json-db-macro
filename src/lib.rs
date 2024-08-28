extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

#[proc_macro_attribute]
pub fn auto_json_db(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields: Fields::Named(fields), .. }) => fields,
        _ => panic!("auto_id macro can only be used on structs with named fields"),
    };

    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

    let expanded: proc_macro2::TokenStream = quote! {
        #[derive(Serialize,Deserialize,Clone,Debug)]
        pub struct #name {
            pub idx: u64, 
            pub created_at:u64,
            #(
                pub #field_names: #field_types,
            )*
        }

        impl #name {
            pub fn new(  #( #field_names: #field_types ),*) -> Self {
                let now_idx = struct_to_json_db::unique_id();
                Self {
                    idx: now_idx.0^now_idx.1,
                    created_at: now_idx.1,
                    #( #field_names, )*
                }
            }
            pub fn get_by_id(id: u64) -> Option<Self> {
                let db = Self::get_all(); 
                db.get(&id).cloned()
            }
            pub fn get_by_ids(ids: &Vec<u64>) -> Vec<Self> {
                let db = Self::get_all(); 
                ids.iter().filter_map(|id| db.get(&id).cloned()).collect()
            }
            pub fn remove_by_id(id: u64){
                let path = DB_STRUCT_JSON_PATH.to_owned()+stringify!(#name)+".json";
                let mut db = Self::get_all(); 
                db.remove(&id);
                let db_string = serde_json::to_string(&db).unwrap();
                struct_to_json_db::write_string_to_txt(&path, db_string);
            }
            pub fn remove_by_ids(ids: &Vec<u64>){
                let path = DB_STRUCT_JSON_PATH.to_owned()+stringify!(#name)+".json";
                let mut db = Self::get_all(); 
                for id in ids{
                    db.remove(&id);
                }
                let db_string = serde_json::to_string(&db).unwrap();
                struct_to_json_db::write_string_to_txt(&path, db_string);
            }
            pub fn get_all()->std::collections::HashMap<u64,Self>{
                let path = DB_STRUCT_JSON_PATH.to_owned()+stringify!(#name)+".json";
                let db_string = struct_to_json_db::read_string_from_txt(&path);
                let db:std::collections::HashMap<u64,Self> = serde_json::from_str(&db_string).unwrap_or_default();
                db
            }
            pub fn clear(){
                let path = DB_STRUCT_JSON_PATH.to_owned()+stringify!(#name)+".json";
                struct_to_json_db::write_string_to_txt(&path, "{}".to_owned());
            }
            pub fn save(&self){
                let path = DB_STRUCT_JSON_PATH.to_owned()+stringify!(#name)+".json";
                let mut db = Self::get_all();
                db.insert(self.idx, self.clone());
                let db_string = serde_json::to_string(&db).unwrap();
                struct_to_json_db::write_string_to_txt(&path, db_string);
            }
            pub fn save_vec(v:Vec<Self>){
                let path = DB_STRUCT_JSON_PATH.to_owned()+stringify!(#name)+".json";
                let mut db = Self::get_all();
                for i in v{
                    db.insert(i.idx, i);
                }
                let db_string = serde_json::to_string(&db).unwrap();
                struct_to_json_db::write_string_to_txt(&path, db_string);
            }
            pub save_all(db:&std::collections::HashMap<u64,Self>){
                let path = DB_STRUCT_JSON_PATH.to_owned()+stringify!(#name)+".json";
                let db_string = serde_json::to_string(db).unwrap();
                struct_to_json_db::write_string_to_txt(&path, db_string);
            }
            pub fn remove(&self){
                Self::remove_by_id(self.idx);
            }
        }
       
    };

    TokenStream::from(expanded)
}

 
extern crate proc_macro;
use std::vec;
use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, DeriveInput, Fields};
use proc_macro2::{Ident, Span};
#[proc_macro_attribute]
pub fn auto_json_db(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    
    let name = &input.ident;
    let name_behalf = Ident::new(&format!("{}_behalf", name), Span::call_site());
     
    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields: Fields::Named(fields), .. }) => fields,
        _ => panic!("auto_id macro can only be used on structs with named fields"),
    };
    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();
     
    let binding = _attr.to_string();
    let attribute_hm = parse_string_to_hashmap(&binding);
    let encript_name = attribute_hm.get("encript").unwrap_or(&"".to_owned()).to_owned();
 
    let mut key_fields_len = 0;
    let mut key_fields: Vec<_> = vec![];
    let mut key_fields_types: Vec<_> = vec![];
    
    if let Some(key_field_string) = attribute_hm.get("key"){
        key_fields = key_field_string.split("|").map(|s| {
            if !s.trim().is_empty() {
                Some(Ident::new(s.trim(), Span::call_site()))
            }else{
                None
            }        
        }  ).collect();
        key_fields_len = key_fields.len();
        key_fields.iter().for_each(|f| {
            if let Some(field_name) = f {
                if let Some(index) = field_names.iter().position(|&r| r.clone().unwrap().to_string() == field_name.to_string() ) {
                    key_fields_types.push(field_types[index].clone());
                }
            }
        });
    }
    let key_format_str = key_fields
            .iter()
            .map(|_| "{}")
            .collect::<Vec<&str>>()
            .join(" ");
    let get_hash_fn = if key_fields_len > 0 {
        quote! {
            pub fn get_hash(  #( #key_fields:& #key_fields_types ),*) -> u64{
                let ukey = format!(#key_format_str, #(  #key_fields ),*);
                 
                return struct_to_json_db::string_to_hash(&ukey.trim().to_lowercase());
            }
        }
    }else{
        quote! {}
    };
     
    let mut unique_field_len = 0;
    let mut unique_field: Vec<_> = vec![];
    let mut unique_field_types: Vec<_> = vec![];
    if let Some(unique_field_string) = attribute_hm.get("unique"){
        unique_field = unique_field_string.split("|").map(|s| {
            if !s.trim().is_empty() {
                Some(Ident::new(s.trim(), Span::call_site()))
            }else{
                None
            }        
        }  ).collect();
        unique_field_len = unique_field.len();
        unique_field.iter().for_each(|f| {
            if let Some(field_name) = f {
                if let Some(index) = field_names.iter().position(|&r| r.clone().unwrap().to_string() == field_name.to_string() ) {
                    unique_field_types.push(field_types[index].clone());
                }
            }
        });
    }
    let mut is_belong_struct = false;
    
 
   if attribute_hm.get("custom_save").is_some(){
        is_belong_struct = true;
    }
 
    let mut is_complex = false;
    if attribute_hm.get("bigsize").is_some(){
        is_complex = true;
    }
    let mut is_singleton = false;
    if attribute_hm.get("singleton").is_some(){
        is_singleton = true;
    }
    let singleton_struct_expand: proc_macro2::TokenStream = quote! {
        #[derive(Serialize,Deserialize,Clone,Debug)]
        pub struct #name {
            pub last_modify:u64,
            #(
                pub #field_names: #field_types,
            )*
        }
        impl #name{
            pub fn new(  #( #field_names: #field_types ),*) -> Self {
                let now_idx = struct_to_json_db::unique_id(); 
                Self {
                    last_modify: now_idx.1,
                    #( #field_names, )*
                }
            }
            pub fn get_path()->String{
                struct_to_json_db::get_struct_json_path()+stringify!(#name)+".json"
            }
          
            pub fn save(&mut self){
                let file_path = Self::get_path();
                let now_idx = struct_to_json_db::unique_id(); 
                self.last_modify = now_idx.1;
                Self::set_data_string(&file_path, serde_json::to_string(self).unwrap());
            }
            pub fn load()->Self{
                let file_path = Self::get_path();
                let db_string = Self::get_data_string(&file_path);
                if let Some(data) = serde_json::from_str(&db_string).ok() {
                    return data;
                }else{
                    return Self::default();
                }
            }
             pub fn from_str(s: &str) -> Option<Self> {
                serde_json::from_str(s).ok()
            }
            pub fn load_raw()->String{
                let file_path = Self::get_path();
                Self::get_data_string(&file_path)
            }
            pub fn set_data_string(file_path:&str,db_string:String){
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::write_string_to_txt_encript(file_path, db_string,&encript);
                        }
                        Err(e) => {
                            struct_to_json_db::write_string_to_txt(file_path, db_string);
                        }
                    }
                }else{
                    struct_to_json_db::write_string_to_txt(file_path, db_string);
                }
            }
            pub fn get_data_string(file_path:&str)->String{
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::read_string_from_txt_encript(&file_path,&encript)
                        }
                        Err(e) => {
                            struct_to_json_db::read_string_from_txt(&file_path)
                        }
                    }
                }else{
                    struct_to_json_db::read_string_from_txt(&file_path)
                }
            }
        }
    };
    let complex_struct_expand: proc_macro2::TokenStream = quote! {
        #[derive(Serialize,Deserialize,Clone,Debug)]
        pub struct #name_behalf{
            pub idx: u64, 
            pub created_at:u64,
            pub last_modify:u64,
            #(
                pub #unique_field: #unique_field_types,
            )*
        }
        impl #name_behalf{
            pub fn new( idx:u64, #( #unique_field: #unique_field_types ),*) -> Self {
                let now_idx = struct_to_json_db::unique_id();
                let mut new_item = Self {
                    idx: idx,
                    created_at:now_idx.1,
                    last_modify:now_idx.1,
                    #( #unique_field,)*
                };
                new_item.idx = new_item.get_unique_hash();
                new_item
            }
             
            pub fn get_path()->String{
                struct_to_json_db::get_struct_json_path()+stringify!(#name)+".json"
            }
            fn get_unique_hash(&self) -> u64 {
                if #key_fields_len > 0 {
                    let ukey = format!(#key_format_str, #(  #key_fields ),*);
                    return struct_to_json_db::string_to_hash(&ukey.trim().to_lowercase());
                } else {
                    return self.idx;
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
                let file_path = Self::get_path();
                let mut db = Self::get_all(); 
                db.remove(&id);
                let db_string = serde_json::to_string(&db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn remove_by_ids(ids: &Vec<u64>){
                let file_path = Self::get_path();
                let mut db = Self::get_all(); 
                for id in ids{
                    
                    db.remove(&id);
                }
                let db_string = serde_json::to_string(&db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
             pub fn from_str(s: &str) -> Option<Self> {
                serde_json::from_str(s).ok()
            }
            pub fn get_all()->std::collections::HashMap<u64,Self>{
                let file_path = Self::get_path();
                let db_string = Self::get_data_string(&file_path);
                let db:std::collections::HashMap<u64,Self> = serde_json::from_str(&db_string).unwrap_or_default();
                db
            }
            pub fn load(&self)->Option< #name >{
                #name ::get_by_id(self.idx) 
            }
            pub fn load_raw()->String{
                let file_path = Self::get_path();
                Self::get_data_string(&file_path)
            }

            pub fn clear(){
                let file_path = Self::get_path();
                struct_to_json_db::write_string_to_txt(&file_path, "".to_owned());
            }
            pub fn save(&self){
                let mut db = Self::get_all();
                db.insert(self.idx, self.clone());
                Self::save_all(&db);
            }
           
            pub fn save_vec(v:Vec<Self>){
                let file_path = Self::get_path();
                let mut db = Self::get_all();
                for i in v{
                    db.insert(i.idx, i);
                }
                let db_string = serde_json::to_string(&db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn save_all(db:&std::collections::HashMap<u64,Self>){
                let file_path = Self::get_path();
                let db_string = serde_json::to_string(db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn remove(&self){
                Self::remove_by_id(self.idx);
            }
            pub fn set_data_string(file_path:&str,db_string:String){
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::write_string_to_txt_encript(file_path, db_string,&encript);
                        }
                        Err(e) => {
                            struct_to_json_db::write_string_to_txt(file_path, db_string);
                        }
                    }
                }else{
                    struct_to_json_db::write_string_to_txt(file_path, db_string);
                }
            }
            pub fn get_data_string(file_path:&str)->String{
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::read_string_from_txt_encript(&file_path,&encript)
                        }
                        Err(e) => {
                            struct_to_json_db::read_string_from_txt(&file_path)
                        }
                    }
                }else{
                    struct_to_json_db::read_string_from_txt(&file_path)
                }
            }
        }
        
        #[derive(Serialize,Deserialize,Clone,Debug)]
        pub struct #name {
            pub idx: u64, 
            pub created_at:u64,
            pub last_modify:u64,
            #(
                pub #field_names: #field_types,
            )*
        }
        impl #name {
            pub fn new(  #( #field_names: #field_types ),*) -> Self {
                let now_idx = struct_to_json_db::unique_id(); 
                
                let mut new_item = Self {
                    idx: now_idx.0^now_idx.1,
                    created_at: now_idx.1,
                    last_modify: now_idx.1,
                    #( #field_names, )*
                };
                new_item.idx = new_item.get_unique_hash();
                new_item
            }
             
            pub fn get_path()->String{
                struct_to_json_db::get_struct_json_path()+stringify!(#name)
            }
            fn get_unique_hash(&self) -> u64 {
                if #key_fields_len > 0 {
                    let ukey = format!(#key_format_str, #(  #key_fields ),*);
                    return struct_to_json_db::string_to_hash(&ukey.trim().to_lowercase());
                } else {
                    return self.idx;
                }
            }
            pub fn load_raw(id: u64)->String{
                let file_path = format!("{}/{}.json",Self::get_path(),id.to_string());
                Self::get_data_string(&file_path)
            }

            pub fn get_by_id(id: u64) -> Option<Self> {
                let file_path = format!("{}/{}.json",Self::get_path(),id.to_string());
                let db_string = Self::get_data_string(&file_path);
                serde_json::from_str(&db_string).ok()
            }
            pub fn get_by_ids(ids: &Vec<u64>) -> Vec<Self> {
                ids.iter().filter_map(|id| Self::get_by_id(id.clone())).collect()
            }
            pub fn remove_by_id(id: u64){
                let file_path = format!("{}/{}.json",Self::get_path(),id.to_string());
                struct_to_json_db::remove_file_by_path(&file_path);
                #name_behalf::remove_by_id(id);
            }
            pub fn remove_by_ids(ids: &Vec<u64>){
               
                ids.iter().for_each(|id|{
                    Self::remove_by_id(id.clone());
                });
            }
            pub fn get_all()->std::collections::HashMap<u64,#name_behalf>{
                let db = #name_behalf::get_all();
                db
            } 
            #get_hash_fn
            pub fn clear(){
                let file_path = Self::get_path()+".json";
                struct_to_json_db::write_string_to_txt(&file_path, "".to_owned());
                struct_to_json_db::remove_all_files_by_path(&Self::get_path());
            }
            pub fn update(&mut self)->Option<u64>{
                let mut exist_item:Vec<u64> = vec![];
                let db = #name_behalf::get_all();
                if #unique_field_len > 0 {
                    exist_item = db.values().filter(|item| {
                        #( self. #unique_field == item. #unique_field && )* true 
                    }).map(|item| item.idx).collect(); 
                }
                if exist_item.len() > 0{
                    self.idx = exist_item[0];
                    let file_path = format!("{}/{}.json",Self::get_path(),exist_item[0].to_string()); 
                    Self::set_data_string(&file_path, serde_json::to_string(self).unwrap());
                    return Some(self.idx);
                } 
                None
            }
            pub fn save_or_update(&mut self)->Option<u64>{
                let mut idx = self.save();
                if idx.is_none(){
                    idx = self.update();
                }
                return idx;
            }
            pub fn save(&self)->Option<u64>{
                let item = Self::get_by_id(self.idx);
                let bh = self.behalf();
                let folder_path = Self::get_path();
                struct_to_json_db::make_folder_if_not_exist(&folder_path);
                
                if item.is_none() {
                    let mut exists = false;
                    let mut db = #name_behalf::get_all();
                    if #unique_field_len > 0 {
                        exists = db.values().any(|item| {
                            #( self. #unique_field == item. #unique_field && )* true 
                        }); 
                    }
                    if exists{
                        return None;
                    }else{
                        let file_path = format!("{}/{}.json",Self::get_path(),self.idx.to_string());  
                        Self::set_data_string(&file_path, serde_json::to_string(self).unwrap());
                        bh.save(); 
                        return Some(self.idx);
                    }
                }else{ 
                    let file_path = format!("{}/{}.json",Self::get_path(),self.idx.to_string());   
                    Self::set_data_string(&file_path, serde_json::to_string(self).unwrap());
                    bh.save();
                    return Some(self.idx);
                }
            }
            #(
                pub fn #unique_field(value: &#unique_field_types)-> Option<Self>{
                    let db = #name::get_all();
                    let rel:Vec<Self> = db.values().filter(|item| &item. #unique_field == value).map(|item| Self::get_by_id(item.idx).unwrap()).collect();
                    if rel.len()>0{
                        return Some(rel[0].clone()); 
                    }
                    None
                }
            )*
            pub fn behalf(&self)->#name_behalf{
                #name_behalf::new(self.idx  #(, self. #unique_field .clone() )*)
            }
            pub fn save_vec(v:Vec<Self>){
                v.into_iter().for_each(|item|{
                    item.save();
                });
            }
            pub fn save_all(db:&std::collections::HashMap<u64,Self>){
                db.iter().for_each(|(idx, item)| {
                    item.save();
                });
            }
            pub fn remove(&self){
                Self::remove_by_id(self.idx);
            }
            pub fn set_data_string(file_path:&str,db_string:String){
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::write_string_to_txt_encript(file_path, db_string,&encript);
                        }
                        Err(e) => {
                            struct_to_json_db::write_string_to_txt(file_path, db_string);
                        }
                    }
                }else{
                    struct_to_json_db::write_string_to_txt(file_path, db_string);
                }
            }
            pub fn get_data_string(file_path:&str)->String{
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::read_string_from_txt_encript(&file_path,&encript)
                        }
                        Err(e) => {
                            struct_to_json_db::read_string_from_txt(&file_path)
                        }
                    }
                }else{
                    struct_to_json_db::read_string_from_txt(&file_path)
                }
            }
        }
       
    };

     let default_struct_expand_with_belong: proc_macro2::TokenStream = quote! {
        #[derive(Serialize,Deserialize,Clone,Debug)]
        pub struct #name {
            pub idx: u64, 
            pub created_at:u64,
            pub last_modify:u64,
            pub ajdb_belong_id:String,
            #(
                pub #field_names: #field_types,
            )*
        }
        impl #name {
            pub fn new( belong_id:&str, #( #field_names: #field_types ),*) -> Self {
                let now_idx = struct_to_json_db::unique_id(); 
                 
                let mut new_item = Self {
                    idx: now_idx.0^now_idx.1,
                    created_at: now_idx.1,
                    last_modify: now_idx.1,
                    ajdb_belong_id: belong_id.to_owned(),
                    #( #field_names, )*
                };
                new_item.idx = new_item.get_unique_hash();
                new_item
            }

            pub fn get_path(belong_id:&str)->String{
                format!("{}{}/{}.json",struct_to_json_db::get_struct_json_path(),stringify!(#name),belong_id)
            }
            pub fn load_raw(belong_id:&str)->String{
                let file_path =  Self::get_path(belong_id);
                Self::get_data_string(&file_path)
            }
            fn get_unique_hash(&self) -> u64 {
                if #key_fields_len > 0 {
                    let ukey = format!(#key_format_str, #( &self. #key_fields ),*);
                   
                    return struct_to_json_db::string_to_hash(&ukey.trim().to_lowercase());
                } else {
                    return self.idx;
                }
            }
            
            #get_hash_fn

            pub fn get_by_id(belong_id:&str,id: u64) -> Option<Self> {
                let db = Self::get_all(belong_id); 
                db.get(&id).cloned()
            }
            pub fn get_by_ids(belong_id:&str,ids: &Vec<u64>) -> Vec<Self> {
                let db = Self::get_all(belong_id); 
                ids.iter().filter_map(|id| db.get(&id).cloned()).collect()
            }
          pub fn remove_by_id(belong_id:&str,id: u64){
                let file_path = Self::get_path(belong_id);
                let mut db = Self::get_all(belong_id); 
                db.remove(&id);
                let db_string = serde_json::to_string(&db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn remove_by_ids(belong_id:&str, ids: &Vec<u64>){
                let mut db = Self::get_all(belong_id); 
                for id in ids.iter(){
                    db.remove(id);
                }
                let file_path = Self::get_path(belong_id);
                let db_string = serde_json::to_string(&db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn get_all(belong_id:&str)->std::collections::HashMap<u64,Self>{
                let file_path =  Self::get_path(belong_id);
              
                let db_string = Self::get_data_string(&file_path);
                 
                let db:std::collections::HashMap<u64,Self> = serde_json::from_str(&db_string).unwrap_or_default();
                db
            }
            #(
                pub fn #unique_field(belong_id:&str,value: &#unique_field_types)-> Option<Self>{
                    let db = #name::get_all(belong_id);
                    let rel:Vec<Self> = db.values().filter(|item| &item. #unique_field == value).map(|item| Self::get_by_id(item.idx).unwrap()).collect();
                    if rel.len()>0{
                        return Some(rel[0].clone()); 
                    }
                    None
                }
            )*
            pub fn clear(belong_id:&str){
                let file_path =  Self::get_path(belong_id);
                struct_to_json_db::write_string_to_txt(&file_path, "".to_owned());
            }
            pub fn update(&mut self)->Option<u64>{
                let mut exist_item:Vec<u64> = vec![];
                let mut db = Self::get_all(&self.ajdb_belong_id);
                if #unique_field_len > 0 {
                    exist_item = db.values().filter(|item| {
                        #( self. #unique_field == item. #unique_field && )* true 
                    }).map(|item| item.idx).collect(); 
                }
                if exist_item.len() > 0{
                    self.idx = exist_item[0];
                    db.insert(self.idx, self.clone());
                    Self::save_all(&self.ajdb_belong_id, &db);
                    return Some(self.idx);
                } 
                None
            }
            pub fn save_or_update(&mut self)->Option<u64>{
                let mut idx = self.save();
                if idx.is_none(){
                    idx = self.update();
                }
                return idx;
            }
            pub fn save(&self)->Option<u64>{
                let mut db = Self::get_all(&self.ajdb_belong_id);
                let idx = self.idx;
                let item_idx:u64 = db.get(&idx).map(|item|item.idx).unwrap_or(0);
                let mut exists = false;
                if idx == item_idx{
                    //update struct
                    db.insert(self.idx, self.clone());
                    Self::save_all(&self.ajdb_belong_id, &db);
                    return Some(idx);
                }else{
                    //insert struct
                    if #unique_field_len > 0 {
                        exists = db.values().any(|item| {
                            #( self. #unique_field == item. #unique_field && )* true 
                        }); 
                    }
                    if exists{
                        return None;
                    }else{
                        db.insert(self.idx, self.clone());
                        Self::save_all(&self.ajdb_belong_id, &db);
                        return Some(idx);
                    }
                }
            }
           
            pub fn save_all(belong_id:&str, db:&std::collections::HashMap<u64,Self>){
                let folder_path = format!("{}/{}",struct_to_json_db::get_struct_json_path(),stringify!(#name));
                struct_to_json_db::make_folder_if_not_exist(&folder_path);
                
                let file_path = Self::get_path(belong_id);
                let db_string = serde_json::to_string(db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn save_vec(belong_id:&str,v:Vec<Self>){
                let file_path = Self::get_path(belong_id.clone());
                let mut db = Self::get_all(belong_id);
                for i in v{
                    db.insert(i.idx, i);
                }
                let db_string = serde_json::to_string(&db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }

            pub fn remove(&self){
                Self::remove_by_id(&self.ajdb_belong_id,self.idx);
            }
            pub fn set_data_string(file_path:&str,db_string:String){
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::write_string_to_txt_encript(file_path, db_string,&encript);
                        }
                        Err(e) => {
                            struct_to_json_db::write_string_to_txt(file_path, db_string);
                        }
                    }
                }else{
                    struct_to_json_db::write_string_to_txt(file_path, db_string);
                }
            }
            pub fn get_data_string(file_path:&str)->String{
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::read_string_from_txt_encript(&file_path,&encript)
                        }
                        Err(e) => {
                            struct_to_json_db::read_string_from_txt(&file_path)
                        }
                    }
                }else{
                    struct_to_json_db::read_string_from_txt(&file_path)
                }
            }
        }
       
    };
    
    let default_struct_expand: proc_macro2::TokenStream = quote! {
        #[derive(Serialize,Deserialize,Clone,Debug)]
        pub struct #name {
            pub idx: u64, 
            pub created_at:u64,
            pub last_modify:u64,
            
            #(
                pub #field_names: #field_types,
            )*
        }
        impl #name {
            pub fn new( #( #field_names: #field_types ),*) -> Self {
                let now_idx = struct_to_json_db::unique_id(); 
                
                let mut new_item = Self {
                    idx: now_idx.0^now_idx.1,
                    created_at: now_idx.1,
                    last_modify: now_idx.1,
                    #( #field_names, )*
                };
                new_item.idx = new_item.get_unique_hash();
                new_item
            }
           
            pub fn get_path()->String{
                struct_to_json_db::get_struct_json_path()+stringify!(#name)+".json"
            }

            pub fn load_raw()->String{
                let file_path = Self::get_path();
                Self::get_data_string(&file_path)
            } 
            fn get_unique_hash(&self) -> u64 {
                if #key_fields_len > 0 {
                    let ukey = format!(#key_format_str, #( &self. #key_fields ),*);
                   
                    return struct_to_json_db::string_to_hash(&ukey.trim().to_lowercase());
                } else {
                    return self.idx;
                }
            }
            
            #get_hash_fn

            pub fn get_by_id(id: u64) -> Option<Self> {
                let db = Self::get_all(); 
                db.get(&id).cloned()
            }
            pub fn get_by_ids(ids: &Vec<u64>) -> Vec<Self> {
                let db = Self::get_all(); 
                ids.iter().filter_map(|id| db.get(&id).cloned()).collect()
            }
            pub fn remove_by_id(id: u64){
                let file_path = Self::get_path();
                let mut db = Self::get_all(); 
                db.remove(&id);
                let db_string = serde_json::to_string(&db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn remove_by_ids(ids: &Vec<u64>){
                let mut db = Self::get_all(); 
                for id in ids.iter(){
                    db.remove(id);
                }
                let file_path = Self::get_path();
                let db_string = serde_json::to_string(&db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn from_str(s: &str) -> Option<Self> {
                serde_json::from_str(s).ok()
            }
            pub fn get_all()->std::collections::HashMap<u64,Self>{
                let file_path = Self::get_path();
                let db_string = Self::get_data_string(&file_path);
                let db:std::collections::HashMap<u64,Self> = serde_json::from_str(&db_string).unwrap_or_default();
                db
            }
            #(
                pub fn #unique_field(value: &#unique_field_types)-> Option<Self>{
                    let db = #name::get_all();
                    let rel:Vec<Self> = db.values().filter(|item| &item. #unique_field == value).map(|item| Self::get_by_id(item.idx).unwrap()).collect();
                    if rel.len()>0{
                        return Some(rel[0].clone()); 
                    }
                    None
                }
            )*
            pub fn clear(){
                let file_path = Self::get_path();
                struct_to_json_db::write_string_to_txt(&file_path, "".to_owned());
            }
            pub fn update(&mut self)->Option<u64>{
                let mut exist_item:Vec<u64> = vec![];
                let mut db = Self::get_all();
                if #unique_field_len > 0 {
                    exist_item = db.values().filter(|item| {
                        #( self. #unique_field == item. #unique_field && )* true 
                    }).map(|item| item.idx).collect(); 
                }
                if exist_item.len() > 0{
                    self.idx = exist_item[0];
                    db.insert(self.idx, self.clone());
                    Self::save_all(&db);
                    return Some(self.idx);
                } 
                None
            }
            pub fn save_or_update(&mut self)->Option<u64>{
                let mut idx = self.save();
                if idx.is_none(){
                    idx = self.update();
                }
                return idx;
            }
            pub fn save(&self)->Option<u64>{
                let mut db = Self::get_all();
                let idx = self.idx;
                let item_idx:u64 = db.get(&idx).map(|item|item.idx).unwrap_or(0);
                let mut exists = false;
                if idx == item_idx{
                    //update struct
                    db.insert(self.idx, self.clone());
                    Self::save_all(&db);
                    return Some(idx);
                }else{
                    //insert struct
                    if #unique_field_len > 0 {
                        exists = db.values().any(|item| {
                            #( self. #unique_field == item. #unique_field && )* true 
                        }); 
                    }
                    if exists{
                        return None;
                    }else{
                        db.insert(self.idx, self.clone());
                        Self::save_all(&db);
                        return Some(idx);
                    }
                }
            }
            pub fn save_vec(v:Vec<Self>){
                let file_path = Self::get_path();
                let mut db = Self::get_all();
                for i in v{
                    db.insert(i.idx, i);
                }
                let db_string = serde_json::to_string(&db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn save_all(db:&std::collections::HashMap<u64,Self>){
                let file_path = Self::get_path();
 
                let db_string = serde_json::to_string(db).unwrap();
                Self::set_data_string(&file_path, db_string);
            }
            pub fn remove(&self){
                Self::remove_by_id(self.idx);
            }
            pub fn set_data_string(file_path:&str,db_string:String){
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::write_string_to_txt_encript(file_path, db_string,&encript);
                        }
                        Err(e) => {
                            struct_to_json_db::write_string_to_txt(file_path, db_string);
                        }
                    }
                }else{
                    struct_to_json_db::write_string_to_txt(file_path, db_string);
                }
            }
            pub fn get_data_string(file_path:&str)->String{
                if #encript_name != ""{
                    match std::env::var(#encript_name) {
                        Ok(encript) => {
                            struct_to_json_db::read_string_from_txt_encript(&file_path,&encript)
                        }
                        Err(e) => {
                            struct_to_json_db::read_string_from_txt(&file_path)
                        }
                    }
                }else{
                    struct_to_json_db::read_string_from_txt(&file_path)
                }
            }
        }
       
    };
    if is_singleton{
        return TokenStream::from(singleton_struct_expand);
    }
    if is_complex {
        return TokenStream::from(complex_struct_expand);
    }
    if is_belong_struct {
        return TokenStream::from(default_struct_expand_with_belong);
    }
    return TokenStream::from(default_struct_expand);
    
}


fn parse_string_to_hashmap(input: &str) -> std::collections::HashMap<String, String> {
    // Regex pattern to match key-value pairs
    let re = Regex::new(r#"\s*(\w+)(?:\s*=\s*"([^"]*)")?\s*(?:,|$)"#).unwrap();
    let mut map = std::collections::HashMap::new();
    for cap in re.captures_iter(input) {
        let key = cap[1].to_string();
        let value = if let Some(value_match) = cap.get(2) {
            value_match.as_str().to_string()
        } else {
            "true".to_string()
        };
        map.insert(key, value);
    }
    map
} 
 
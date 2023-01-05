
use actix_multipart::Field;
use actix_multipart::Multipart;
use actix_web::HttpResponse;
use actix_web::http::Error;
use mysql::Pool;
use savaged_libs::user::ImageUpdateResult;
use savaged_libs::websocket_message::SimpleAPIReturn;
use std::fs;
// use std::io::Bytes;
use std::path;
use std::path::Path;
use actix_easy_multipart::tempfile::Tempfile;
use actix_easy_multipart::text::Text;
use actix_easy_multipart::MultipartForm;

use actix_web:: {
    // get,
    post,

    // multipart
    web::Json,
    web::Data,
};
use actix_web::HttpRequest;

use crate::CONFIG_ALLOWED_IMAGE_TYPES;
use crate::utils::encrypt_password;
use crate::utils::image_to_webp;
use crate::utils::resize_image_max;

use super::super::db::users::{
    update_user,
    update_password,
    get_remote_user,
    update_user_login_tokens,
    username_available,
    save_username,
};

use serde::{Serialize, Deserialize};
use savaged_libs::user::{ User, LoginToken, UserUpdateResult };



#[post("/_api/user/save-username")]
pub async fn api_user_save_username (
    pool: Data<Pool>,
    form: Json<UserNameForm>,
    request: HttpRequest,
) -> Json<bool> {


    // println!("api_user_save_username called");

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    let mut username = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
        }
        None => {}
    }
    match &form.username {
        Some( val ) => {
            username = val.to_owned();
        }
        None => {}
    }

    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match user_option {
        Some( user ) => {
            // println!("api_user_username_available called {}", &username);
            let result_count = save_username( pool.clone(), user, username.clone() );
            // println!("api_user_username_available result {}", &result_count);
            if result_count == 1 {
                return Json( true );
            }
        }
        None => {}
    }
    return Json(false);

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct UserNameForm {
    username: Option<String>,
    api_key: Option<String>,
    login_token: Option<String>,
}

#[post("/_api/user/username-available")]
pub async fn api_user_username_available (
    pool: Data<Pool>,
    form: Json<UserNameForm>,
    request: HttpRequest,
) -> Json<bool> {

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    let mut username = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
        }
        None => {}
    }
    match &form.username {
        Some( val ) => {
            username = val.to_owned();
        }
        None => {}
    }

    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match user_option {
        Some( user ) => {
            // println!("api_user_username_available called {}", &username);
            return Json( username_available( pool.clone(), user, username.clone() ));
        }
        None => {}
    }
    return Json(false);
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateTokenNameForm {
    api_key: Option<String>,
    login_token: Option<String>,
    selected_token: Option<String>,
    new_value: Option<String>,
}

#[post("/_api/user/token-update-name")]
pub async fn api_user_token_update_name(
    pool: Data<Pool>,
    form: Json<UpdateTokenNameForm>,
    request: HttpRequest,
) -> Json< Vec<LoginToken> > {

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    let mut selected_token = "".to_owned();
    let mut new_value = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
        }
        None => {}
    }
    match &form.selected_token {
        Some( val ) => {
            selected_token = val.to_owned();
        }
        None => {}
    }
    match &form.new_value {
        Some( val ) => {
            new_value = val.to_owned();
        }
        None => {}
    }

    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match user_option {
        Some( user ) => {

            let mut return_tokens = user.login_tokens.clone();

            for token in &mut return_tokens {
                if token.token == selected_token {
                    token.friendly_name = new_value.to_owned();
                }
            }

            update_user_login_tokens( pool.clone(), user.id, return_tokens.clone() );

            return Json( return_tokens.clone() );
        }
        None => {

        }
    }

    return Json( Vec::new() );
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateSettingData {
    api_key: Option<String>,
    login_token: Option<String>,
    password: Option<String>,
    repeat_password: Option<String>,
    remove_image: bool,
    current_user: String,
}

#[post("/_api/user/update-settings")]
pub async fn api_user_update_settings(
    pool: Data<Pool>,
    form: Json<UpdateSettingData>,
    request: HttpRequest,
) -> Json< UserUpdateResult > {

    let mut return_value: UserUpdateResult = UserUpdateResult {
        success: false,
        password_changed: false,
        message: "".to_string(),
    };

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;

    // let mut new_value = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
        }
        None => {}
    }
    // match &form.selected_token {
    //     Some( val ) => {
    //         selected_token = val.to_owned();
    //     }
    //     None => {}
    // }

    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match user_option {
        Some( user ) => {

            let user_data: Result<User, serde_json::Error> = serde_json::from_str( &form.current_user );
            match user_data {
                Ok(mut user_settings) => {
                    // println!("api_user_update_settings() user found!");
                    // println!("api_user_update_settings() user_data {:?}", user_settings);
                    // println!("api_user_update_settings() form.password {:?}", form.password);
                    // println!("api_user_update_settings() form.repeat_password {:?}", form.repeat_password);
                    // println!("api_user_update_settings() form.remove_image {:?}", form.remove_image);

                    // Override any potential hacker variables in POST
                    user_settings.is_premium = user.is_premium;
                    user_settings.is_ace = user.is_ace;
                    user_settings.is_admin = user.is_admin;
                    user_settings.is_developer = user.is_developer;
                    user_settings.id = user.id;
                    user_settings.lc_wildcard_reason = user.lc_wildcard_reason;
                    user_settings.premium_expires = user.premium_expires;
                    user_settings.last_seen_ip = user.last_seen_ip;
                    user_settings.last_seen_on = user.last_seen_on;


                    let data_dir_path = "./data/uploads/";
                    let png_filename = data_dir_path.to_owned() + &"users/".to_owned() + &user_settings.id.to_string()  + &".png".to_owned();
                    let jpg_filename = data_dir_path.to_owned() + &"users/".to_owned() + &user_settings.id.to_string()  + &".jpg".to_owned();
                    let webp_filename = data_dir_path.to_owned() + &"users/".to_owned() + &user_settings.id.to_string()  + &".webp".to_owned();

                    if form.remove_image {
                        if std::path::Path::new(&png_filename).exists() {
                            let _ = fs::remove_file(&png_filename);
                        }
                        if std::path::Path::new(&jpg_filename).exists() {
                            let _ = fs::remove_file(&jpg_filename);
                        }
                        if std::path::Path::new(&webp_filename).exists() {
                            let _ = fs::remove_file(&webp_filename);
                        }
                        user_settings.profile_image = "".to_string();
                    }
                    if std::path::Path::new(&png_filename).exists() {
                        user_settings.profile_image = "png".to_string();
                    }
                    if std::path::Path::new(&jpg_filename).exists() {
                        user_settings.profile_image = "jpg".to_string();
                    }
                    if std::path::Path::new(&webp_filename).exists() {
                        user_settings.profile_image = "webp".to_string();
                    }

                    let mut do_notify_admins = false;
                    if !user_settings.activated {
                        do_notify_admins = true;
                    }


                    if !user_settings.email.is_empty() {
                        let mut new_encrypted_pass: Option<String> = None;
                        if
                            form.password != None && !form.password.as_ref().unwrap().is_empty()
                            && form.repeat_password != None && !form.repeat_password.as_ref().unwrap().is_empty()
                            && form.repeat_password.as_ref() == form.password.as_ref()
                        {
                            return_value.password_changed = true;
                            new_encrypted_pass = Some(encrypt_password( form.password.clone().unwrap().to_owned() ));
                            update_password(
                                pool.clone(),
                                user_settings.clone(),
                                new_encrypted_pass,
                            );
                        }

                        let rows_affected = update_user(
                            pool.clone(),
                            user_settings.clone(),
                        );

                        if rows_affected == 1 {
                            return_value.success = true;
                            return_value.message = "User Updated".to_string();
                        }


                    } else {
                        return_value.message = "Email Address cannot be empty - this might be a data transfer error.".to_string();
                    }
                }
                Err( _err ) => {
                    return_value.message = "No user data sent?".to_string();
                }

            }
        }
        None => {
            return_value.message = "No user found? Are you logged in?".to_string();
        }
    }

    return Json( return_value );
}

#[post("/_api/user/token-remove")]
pub async fn api_user_token_remove(
    pool: Data<Pool>,
    form: Json<UpdateTokenNameForm>,
    request: HttpRequest,
) -> Json< Vec<LoginToken> > {

    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    let mut selected_token = "".to_owned();
    // let mut new_value = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            login_token = Some(val.to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            api_key = Some(val.to_owned());
        }
        None => {}
    }
    match &form.selected_token {
        Some( val ) => {
            selected_token = val.to_owned();
        }
        None => {}
    }
    // match &form.new_value {
    //     Some( val ) => {
    //         new_value = val.to_owned();
    //     }
    //     None => {}
    // }

    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match user_option {
        Some( user ) => {

            let mut return_tokens: Vec<LoginToken> = Vec::new();

            for token in user.login_tokens.iter() {
                if token.token != selected_token {
                    return_tokens.push( token.clone() );
                }
            }

            update_user_login_tokens( pool.clone(), user.id, return_tokens.clone() );

            return Json( return_tokens.clone() );
        }
        None => {

        }
    }

    return Json( Vec::new() );
}

#[derive(MultipartForm)]
pub struct ImageDataForm {
    api_key: Option<Text<String>>,
    login_token: Option<Text<String>>,
    upload_type: Option<Text<String>>,
    crop_square: Option<Text<String>>,
    #[multipart]
    image: Tempfile,
}


#[post("/_api/user/set-user-image-data")]
pub async fn api_user_set_user_image_data(
    pool: Data<Pool>,
    form: MultipartForm<ImageDataForm>,
    // form: Json<ImageDataForm>,
    // mut payload: Multipart,
    request: HttpRequest,
) -> Json< ImageUpdateResult > {

    let mut rv: ImageUpdateResult = ImageUpdateResult {
        success: false,
        message: "Not Authenticated".to_owned(),
        image_url: "".to_owned(),
    };
    println!("api_user_set_user_image_data form called");
    // println!("api_user_set_user_image_data form {:?}", &form);
    // println!("api_user_set_user_image_data payload {:?}", &payload);
    let mut login_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    let mut upload_type: String = "".to_string();
    let mut crop_square: bool = false;

    // let mut new_value = "".to_owned();
    match &form.login_token {
        Some( val ) => {
            // login_token = Some(format!("{:?}", val.deref()));
            login_token = Some(val.as_str().to_owned());
        }
        None => {}
    }
    match &form.api_key {
        Some( val ) => {
            // api_key = Some(val.to_owned());
            api_key = Some(val.as_str().to_owned());
        }
        None => {}
    }
    match &form.upload_type {
        Some( val ) => {
            // upload_type = val.to_owned();
            upload_type = val.as_str().to_owned();
        }
        None => {}
    }
    match &form.crop_square {
        Some( val ) => {
            // upload_type = val.to_owned();
            if !val.as_str().to_owned().is_empty() {
                crop_square = true;
            }
        }
        None => {}
    }
    // match &form.new_value {
    //     Some( val ) => {
    //         new_value = val.to_owned();
    //     }
    //     None => {}
    // }

    let content_type = form
        .image
        .content_type
        .as_ref()
        .map(|m| m.as_ref())
        .unwrap_or("null");
    let file_name = form
        .image
        .file_name
        .as_ref()
        .map(|m| m.as_ref())
        .unwrap_or("null");


    let user_option = get_remote_user(
        pool.clone(),
        api_key,
        login_token,
        request,
    );

    match user_option {
        Some( user ) => {

            // println!("api_user_set_user_image_data login_token {:?}", &login_token);
            // println!("api_user_set_user_image_data upload_type {:?}", &upload_type);
            // println!("api_user_set_user_image_data file_name {}", &file_name);
            // println!("api_user_set_user_image_data content_type {}", &content_type);

            for allowed in CONFIG_ALLOWED_IMAGE_TYPES {
                if allowed == &content_type {


                    let _ = fs::create_dir_all( "/data/uploads/".to_owned() + &"users/" + &user.id.to_string() );
                    // Check if pic actually exists, clear out if not.
                    let mut png_filename = "./data/uploads/".to_owned() + &"users/" + &user.id.to_string() + &"-session-" + &upload_type.as_str() + &".png";
                    let mut jpg_filename = "./data/uploads/".to_owned() + &"users/" + &user.id.to_string() + &"-session-" + &upload_type.as_str() + &".jpg";
                    let mut webp_filename = "./data/uploads/".to_owned() + &"users/" + &user.id.to_string() + &"-session-" + &upload_type.as_str() + &".webp";
                    png_filename = png_filename.replace("-session-user", "");
                    jpg_filename = jpg_filename.replace("-session-user", "");
                    webp_filename = webp_filename.replace("-session-user", "");


                    if Path::new(&png_filename.as_str()).is_file() {
                        let _ = fs::remove_file(&png_filename);
                    }
                    if Path::new(&jpg_filename.as_str()).is_file() {
                        let _ = fs::remove_file(&jpg_filename);
                    }
                    if Path::new(&webp_filename.as_str()).is_file() {
                        let _ = fs::remove_file(&webp_filename);
                    }

                    let mut save_file_name = webp_filename.to_owned();

                    if &content_type == &"image/png" {
                        save_file_name = png_filename.to_owned();
                        let _ = fs::copy(form.image.file.path().to_str().unwrap(), &save_file_name);
                        let _ = image_to_webp( &png_filename, &webp_filename, 1000, crop_square );


                        let _ = fs::remove_file(&png_filename);
                    } else if &content_type == &"image/jpg" || &content_type == &"image/jpeg" {
                        save_file_name = jpg_filename.to_owned();
                        let _ = fs::copy(form.image.file.path().to_str().unwrap(), &save_file_name);

                        let _ = image_to_webp( &jpg_filename, &webp_filename, 1000, crop_square );

                        let _ = fs::remove_file(&jpg_filename);
                    } else {
                        let _ = fs::copy(form.image.file.path().to_str().unwrap(), &save_file_name);
                        let _ = resize_image_max( &webp_filename, 1000, crop_square );
                    }



                    // println!("api_user_set_user_image_data content_type 2 {}", &content_type);
                    // println!("api_user_set_user_image_data png_filename {}", &png_filename);
                    // println!("api_user_set_user_image_data jpg_filename {}", &jpg_filename);
                    // println!("api_user_set_user_image_data webp_filename {}", &webp_filename);
                    // println!("api_user_set_user_image_data save_file_name {}", &save_file_name);
                    rv.success = true;
                    rv.message = "Uploaded".to_owned();
                    rv.image_url = webp_filename.replace("/data/uploads/", "/data-images/");
                }
            }
            rv.message = "Cannot upload image, only jpg, jpeg, or png files are allowed.".to_owned();
        }
        None => {

        }
    }

    return Json( rv );
    // Ok(HttpResponse::Ok().into())
}
/*


router.post(CONFIGApiPrefix + '/user/set-user-image-data', async (req: express.Request, res: express.Response, next: any) => {
    let userObj = await getAPIUser( req );
    if( process.env.VERBOSE ) {
        console.info( req.url, userObj ? userObj.id : 0, userObj ? userObj.username : "anon" )
    }
    if(
        req.body.type
        && req.files
        && req.files.image
        && userObj
    ) {
        let dataDirPath: string = app.get('dataDirPath');

        if( dataDirPath ) {

            let saveImage: UploadedFile = req.files.image as UploadedFile;
            console.log("set-user-image-data req.files.image", req.files.image);
            // Check if is a valid upload file
            if( CONFIGAllowedImageTypes.indexOf( saveImage.mimetype ) > -1) {

                // Check if pic actually exists, clear out if not.
                let png_filename = dataDirPath + "users/" + userObj.id + "-session-" + req.body.type +".png";
                let jpg_filename = dataDirPath + "users/" + userObj.id + "-session-" + req.body.type +".jpg";
                let webp_filename = dataDirPath + "users/" + userObj.id + "-session-" + req.body.type +".webp";
                png_filename = png_filename.replace("-session-user", "");
                jpg_filename = jpg_filename.replace("-session-user", "");
                webp_filename = webp_filename.replace("-session-user", "");

                // if( req.body.type == "user") {
                //     png_filename = dataDirPath + "users/" + userObj.id + "-session-" + req.body.type +".png";
                //     jpg_filename = dataDirPath + "users/" + userObj.id + "-session-" + req.body.type +".jpg";
                //     png_filename = png_filename.replace("-session-user", "");
                //     jpg_filename = jpg_filename.replace("-session-user", "");
                // }

                if ( await fs.existsSync(png_filename) ) {
                    fs.unlinkSync( png_filename );
                }
                if ( await fs.existsSync(jpg_filename) ) {
                    fs.unlinkSync( jpg_filename );
                }
                if ( await fs.existsSync(webp_filename) ) {
                    fs.unlinkSync( webp_filename );
                }

                let save_file_name = jpg_filename;

                if(  saveImage.mimetype == "image/png" ) {
                    save_file_name = png_filename
                }
                if(  saveImage.mimetype == "image/webp" ) {
                    save_file_name = webp_filename
                }

                await sharp(saveImage.data)
                .withMetadata()
                .webp()
                .toFile(webp_filename);

                save_file_name = webp_filename;
                console.log("saveImage.mimetype", saveImage.mimetype);
                let image_url = save_file_name.replace( dataDirPath, CONFIGDataGetUrlPrefix + "/");

                console.log("replace image", req.body.type, saveImage.mimetype, save_file_name, CONFIGImageHeightMax, CONFIGImageWidthMax)
                if( req.body.type == "user") {
                    await sharp(saveImage.data)
                    .resize(
                        CONFIGImageHeightMax,
                        CONFIGImageWidthMax,
                        {
                            fit: sharp.fit.cover,
                            // withoutEnlargement: true,
                        }
                    )
                    .toFile(save_file_name);
                } else {
                    await sharp(saveImage.data)
                    .resize(CONFIGImageHeightMax, CONFIGImageWidthMax,
                        {
                      fit: sharp.fit.inside,
                      withoutEnlargement: true,
                    })
                    .toFile(save_file_name);
                }

                res.json( { "success": true, "message": "Uploaded", "image_url": image_url } );
                return;
            } else {

                res.json( { "success": false, "message": "Cannot upload image, only jpg, jpeg, or png files are allowed." } );
                return;
            }

        } else {

            res.json( { "success": false, "message": "Cannot upload image, only jpg, jpeg, or png files are allowed." } );
            return;
        }
    } else {

        res.json( { "success": false, "message": "Internal Server Error" } );
        return;
    }
});
 */

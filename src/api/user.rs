
use mysql::Pool;
use savaged_libs::websocket_message::SimpleAPIReturn;
use std::fs;
use std::path;

use actix_web:: {
    // get,
    post,
    web::Json,
    web::Data,
};
use actix_web::HttpRequest;

use crate::utils::encrypt_password;

use super::super::db::users::{
    update_user,
    update_password,
    get_remote_user,
    update_user_login_tokens,
};

use serde::{Serialize, Deserialize};
use savaged_libs::user::{ User, LoginToken, UserUpdateResult };



#[post("/_api/user/save-username")]
pub async fn api_user_save_username (
    pool: Data<Pool>,
    // form: Json<LoginForm>,
    request: HttpRequest,
) -> Json<bool> {


    println!("api_user_save_username called");
    return Json(false);

}

#[post("/_api/user/username-available")]
pub async fn api_user_username_available (
    pool: Data<Pool>,
    // form: Json<LoginForm>,
    request: HttpRequest,
) -> Json<bool> {
    println!("api_user_username_available called");
    return Json(false);
}

#[post("/_api/user/set-user-image-data")]
pub async fn api_user_user_image_data (
    pool: Data<Pool>,
    // form: Json<LoginForm>,
    request: HttpRequest,
) -> Json<SimpleAPIReturn> {

    println!("api_user_user_image_data called");

    let rv = SimpleAPIReturn {
        success: false,
        message: "Not implemented".to_owned(),
    };

    return Json(rv);
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

                        update_user(
                            pool.clone(),
                            user_settings.clone(),
                        );


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

/*

router.post(CONFIGApiPrefix + '/user/update-settings', async (req: express.Request, res: express.Response, next: any) => {
    let apiUser = await getAPIUser( req );
    if(
        req.session
        && apiUser
        && process.env.SHA_SECRET_KEY
    ) {
        if( req.body  ) {
            let newPassword: string = "";
            let updatedPassword: boolean = false;
            if(
                req.body.password
                && req.body.repeat_password
                && req.body.repeat_password == req.body.password
            ) {
                updatedPassword = true;
                let hash = crypto.createHash('sha224' );

                let data = hash.update( utf8.encode(req.body.password));
                data = hash.update( utf8.encode(process.env.SHA_SECRET_KEY) );
                newPassword= data.digest('base64');
            }

            let userGroups = await DB.UserGroups.all();

            let userObj = new User(apiUser.export(), userGroups );
            let userObjOriginal = new User(userObj.export(), userGroups );

            userObj.importObj( req.body, userGroups );

            console.log("req.body", req.body)
            // Override any potential hacker variables in POST
            userObj.isWildcard = userObjOriginal.isWildcard;
            userObj.isAce = userObjOriginal.isAce;
            userObj.isAdmin = userObjOriginal.isAdmin;
            userObj.isDeveloper = userObjOriginal.isDeveloper;
            userObj.id = userObjOriginal.id;
            userObj.lifetimeWildCard = userObj.lifetimeWildCard ;
            userObj.lifetimeWildcardReason = userObj.lifetimeWildcardReason ;
            userObj.wildcardExpires = userObj.wildcardExpires;
            userObj.lastSeenIP = userObj.lastSeenIP;
            userObj.lastSeenOn = userObj.lastSeenOn;

            let dataDirPath: string = app.get('dataDirPath');
            let png_filename = dataDirPath + "users/" + userObj.id + ".png";
            let jpg_filename = dataDirPath + "users/" + userObj.id + ".jpg";
            let webp_filename = dataDirPath + "users/" + userObj.id + ".webp";

            if( req.body.remove_image  ) {

                if ( await fs.existsSync(png_filename) ) {
                    fs.unlinkSync( png_filename );
                }
                if ( await fs.existsSync(jpg_filename) ) {
                    fs.unlinkSync( jpg_filename );
                }
                if ( await fs.existsSync(webp_filename) ) {
                    fs.unlinkSync( webp_filename );
                }
                userObj.profileImage = "";
            }

            if ( await fs.existsSync(png_filename) ) {
                userObj.profileImage = "png"
            }
            if ( await fs.existsSync(jpg_filename) ) {
                userObj.profileImage = "jpg"
            }
            if ( await fs.existsSync(webp_filename) ) {
                userObj.profileImage = "webp"
            }

            let doNotifyAdmins = false;
            if( !userObj.activated  ) {
                doNotifyAdmins = true;
            }

            if( userObj.email && userObj.email.trim() ) {

                await DB.Users.update_user(
                    userObj,
                    newPassword
                );

                if( newPassword.length > 0 ) {
                    sendStandardEmail(
                        userObj.id,
                        "Password Change Notification",
                        "<p>This is just a little notification to inform you that your password at " + CONFIGSiteTitle + " (<a href=\"" + CONFIGLiveHost+ "\">" + CONFIGLiveHost + "</a>) has been changed.</p>",
                    );
                }

                let current_user = await DB.Users.get_user( userObj.id );
                if( current_user ) {
                    let dataDirPath: string = app.get('dataDirPath');

                    // Check if profile pic actually exists, clear out profile_image if not.
                    if( dataDirPath ) {

                        if ( await fs.existsSync(dataDirPath + "users/" + current_user.id + ".jpg")) {
                            current_user.profile_image = "jpg";
                        } else if ( await fs.existsSync(dataDirPath + "users/" + current_user.id + ".png")) {
                            current_user.profile_image = "png";
                        } else {
                            current_user.profile_image = "";
                        }
                    }
                    let sess = req.session;
                    sess['current_user'] = current_user;
                    delete req.session["current_user"]["password"];
                    sess.save( () => {

                        // res.json( { "success": true, "user": current_user, "message": "Settings Saved", "password_changed": false } );
                        res.json( { "success": true, "user": current_user, "message": "User Updated", "password_changed": updatedPassword } );

                        // res.setHeader('Content-Type', 'application/json');

                        // res.send( JSON.stringify(true) )
                        // return;
                    })
                }
            } else {
                res.json( { "success": false, "user": null, "message": "Email Address cannot be empty - this might be a data transfer error.", "password_changed": false } );
                // return;
            }

            // return;
        } else {
            res.json( { "success": false, "user": null, "message": "No Data", "password_changed": false } );
            // return;
        }
    } else {
        res.json( { "success": false, "user": null, "message": "Not Logged In", "password_changed": false } );
        // return;
    }
});

 */

 /*

router.post(CONFIGApiPrefix + '/user/username-available', async (req: express.Request, res: express.Response, next: any) => {
    let userObj = await getAPIUser( req );
    if( process.env.VERBOSE ) {
        console.info( req.url, userObj ? userObj.id : 0, userObj ? userObj.username : "anon" )
    }
    if(
        req.body.username
        && userObj
    ) {

        let username = normalizeUsername( req.body.username );

        if( username.trim() ) {
            let available = await DB.Users.username_available(username, userObj.id );

            if( available ) {
                res.send( true )
                return;
            } else {
                res.send( false )
                return;
            }
        } else {
            res.send( false )
            return;
        }
    } else {
        res.send( false )
        return;
    }
});

router.post(CONFIGApiPrefix + '/user/save-username', async (req: express.Request, res: express.Response, next: any) => {
    let userObj = await getAPIUser( req );
    if( process.env.VERBOSE ) {
        console.info( req.url, userObj ? userObj.id : 0, userObj ? userObj.username : "anon" )
    }
    if(
        req.body.username
        && userObj
    ) {

        let username = normalizeUsername( req.body.username );
        if( username.trim() ) {
            let affectedRows = await DB.Users.save_username( username, userObj.id );

            if( affectedRows > 0 ) {
                res.send( true )
                return;
            } else {
                res.send( false )
                return;
            }
        } else {
            res.send( false )
            return;
        }

    } else {
        res.send( false )
        return;
    }

});

router.post(CONFIGApiPrefix + '/auth/set-user-image-data', async (req: express.Request, res: express.Response, next: any) => {
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
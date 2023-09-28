use crate::db::users::get_user;
use actix_web::web::Data;
use image::io::Reader as ImageReader;
use image::{DynamicImage, EncodableLayout, GenericImage, GenericImageView, ImageFormat}; // Using image crate: https://github.com/image-rs/image
use lettre::transport::smtp::authentication::Credentials;
use lettre::{
    message::{header, MultiPart, SinglePart},
    Message, SmtpTransport, Transport,
};
use mysql_async::Pool;
use sha2::{Digest, Sha224};
use voca_rs::strip::strip_tags;
use webp::{Encoder, PixelLayout, WebPImage, WebPMemory}; // Using webp crate: https://github.com/jaredforth/webp

use std::fs::File;
use std::io::Write;

pub fn encrypt_password(password: String) -> String {
    let mut sha_secret_key = "".to_owned();
    match std::env::var("SHA_SECRET_KEY") {
        Ok(val) => {
            sha_secret_key = val.parse().unwrap();
        }
        Err(_) => {}
    }

    let mut hasher = Sha224::new();
    hasher.update(password.to_owned());
    hasher.update(sha_secret_key.to_owned());
    let data = hasher.finalize();
    return format!("b'{}'", base64::encode(data));
}

pub async fn send_standard_email(
    pool: &Data<Pool>,
    user_id: u32,
    subject: String,
    html_message: String,
) -> bool {
    let mut email_from_address = "".to_owned();
    let mut email_from_name = "".to_owned();
    let mut email_subject_prefix = "".to_owned();

    match std::env::var("EMAIL_FROM_ADDRESS") {
        Ok(val) => {
            email_from_address = val.parse().unwrap();
        }
        Err(err) => {
            println!(
                "Error .env EMAIL_FROM_ADDRESS env variable not set! {}",
                err.to_string()
            );
            return false;
        }
    }

    match std::env::var("EMAIL_FROM_NAME") {
        Ok(val) => {
            email_from_name = val.parse().unwrap();
        }
        Err(err) => {
            println!(
                "Error .env EMAIL_FROM_NAME env variable not set! {}",
                err.to_string()
            );
            return false;
        }
    }

    match std::env::var("EMAIL_SUBJECT_PREFIX") {
        Ok(val) => {
            email_subject_prefix = val.parse().unwrap();
        }
        Err(_) => {
            // println!("Error EMAIL_SUBJECT_PREFIX env variable not set!");
            // return false;
        }
    }

    let compound_subject =
        email_subject_prefix.to_string() + &" ".to_string() + &subject.to_string();

    let user_result = get_user(pool, user_id).await;
    match user_result {
        Some(user) => {
            let result = send_email(
                email_from_address,
                email_from_name,
                user.email.to_owned(),
                user.get_real_name().to_owned(),
                compound_subject.trim().to_string(),
                html_message,
                false,
                "".to_string(),
                "".to_string(),
            )
            .await;
            return result;
        }
        None => {}
    }
    return false;
}

pub async fn send_email(
    from: String,
    from_name: String,
    to: String,
    to_name: String,
    subject: String,
    html_message: String,
    admin_footer: bool,
    reply_to: String,
    reply_to_name: String,
) -> bool {
    let mut smtp_host = "".to_string();
    match std::env::var("MAIL_HOST") {
        Ok(val) => {
            smtp_host = val.parse().unwrap();
        }
        Err(_) => {
            println!("Error MAIL_HOST env variable not set!");
            return false;
        }
    }
    let mut smtp_username = "".to_string();
    match std::env::var("MAIL_USERNAME") {
        Ok(val) => {
            smtp_username = val.parse().unwrap();
        }
        Err(_) => {
            println!("Error MAIL_USERNAME env variable not set!");
            return false;
        }
    }

    let mut smtp_password = "".to_string();
    match std::env::var("MAIL_PASSWORD") {
        Ok(val) => {
            smtp_password = val.parse().unwrap();
        }
        Err(_) => {
            println!("Error MAIL_PASSWORD env variable not set!");
            return false;
        }
    }

    let mut smtp_port: u16 = 587;
    match std::env::var("MAIL_PORT") {
        Ok(val) => {
            smtp_port = val.parse().unwrap();
        }
        Err(_) => {
            println!("Error MAIL_PORT env variable not set!");
            return false;
        }
    }

    let mut smtp_secure = false;
    match std::env::var("MAIL_SECURE") {
        Ok(val) => {
            let int_value: u8 = val.parse().unwrap();
            if int_value > 0 {
                smtp_secure = true;
            }
        }
        Err(_) => {
            println!("Error MAIL_SECURE env variable not set!");
            return false;
        }
    }

    let mut stmp_from = from.clone();
    if !from_name.is_empty() {
        stmp_from = from_name + &"<".to_owned() + &from.to_owned() + &">".to_owned();
    }

    let mut stmp_to = to.clone();
    if !to_name.is_empty() {
        stmp_to = to_name + &"<".to_owned() + &to.to_owned() + &">".to_owned();
    }

    let mut email = Message::builder()
        .from(stmp_from.parse().unwrap())
        // .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to(stmp_to.parse().unwrap())
        .subject(subject.clone())
        .multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(String::from(strip_tags(&html_message.clone()))), // Every message should have a plain text fallback.
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(String::from(_standardize_html_email(
                            html_message.clone(),
                            admin_footer,
                        ))),
                ),
        )
        .unwrap();
    if !reply_to.is_empty() {
        let mut reply_to = reply_to.clone();
        if !reply_to_name.is_empty() {
            reply_to = reply_to_name + &"<".to_owned() + &reply_to.to_owned() + &">".to_owned();
        }

        email = Message::builder()
            .from(stmp_from.parse().unwrap())
            .reply_to(reply_to.parse().unwrap())
            .to(stmp_to.parse().unwrap())
            .subject(subject)
            // .body(_standardize_html_email(html_message, admin_footer))
            // .html(_standardize_html_email(html_message, admin_footer))
            .multipart(
                MultiPart::alternative() // This is composed of two parts.
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(String::from(strip_tags(&html_message.clone()))), // Every message should have a plain text fallback.
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(String::from(_standardize_html_email(
                                html_message.clone(),
                                admin_footer,
                            ))),
                    ),
            )
            .unwrap();
    }
    let creds = Credentials::new(smtp_username, smtp_password);

    // if !smtp_port.is_empty() {
    //     smtp_host = smtp_host + &":".to_string() + &smtp_port;
    // }
    // Open a remote connection to gmail
    println!("smtp_secure {}", smtp_secure);
    if smtp_secure {
        let mailer = SmtpTransport::starttls_relay(smtp_host.as_str())
            .unwrap()
            .port(smtp_port)
            .credentials(creds)
            .build();
        match mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => println!("ERROR Could not send email: {:?}", e),
        }
    } else {
        let mailer = SmtpTransport::builder_dangerous(smtp_host.as_str())
            .port(smtp_port)
            .credentials(creds)
            .build();
        match mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => println!("ERROR Could not send email: {:?}", e),
        }
    }

    // Send the email

    // if( !smtp_secure ) {
    //     smtp_secure = false;
    // }

    // let transporter = NodeMailer.createTransport({
    //     host: smtpHost,
    //     port: smtpPort,
    //     secure: smtpSecure, // true for 465, false for other ports
    //     auth: {
    //         user: smtpUsername, // generated ethereal user
    //         pass: smtpPassword // generated ethereal password
    //     }
    // });

    // let recipients: string = "";
    // if( typeof( to) == "string" ) {
    //     recipients = to;
    // } else {
    //     recipients = to.join(", ");
    // }

    // let sender = from;
    // if( fromName ) {
    //     sender = fromName + " <" + from + ">";
    // }

    // let textMessage = SanitizeHTML(htmlMessage);

    // if( replyTo ) {
    //     replyTo = replyToName + " <" + replyTo + ">";
    // } else {
    //     replyTo = sender
    // }

    // try {
    //     let info = await transporter.sendMail({
    //         from: sender, // sender address
    //         replyTo: replyTo, // sender address
    //         to: recipients, // list of receivers
    //         subject: CONFIGEmailSubjectPrefix + subject, // Subject line
    //         text: textMessage, // plain text body
    //         html: _StandardizeHTMLEmail(htmlMessage, adminFooter) // html body
    //     });

    //     return true;
    // }

    // catch( error ) {
    //     console.error(   "Server.utils.sendEmail(): Exception: couldn't send email!", error)
    //     return false;
    // }

    return false;
}

fn _standardize_html_email(incoming_message: String, admin_unsubscribe: bool) -> String {
    let mut config_site_title = "".to_string();
    match std::env::var("SITE_TITLE") {
        Ok(val) => {
            config_site_title = val.parse().unwrap();
        }
        Err(_) => {
            println!("Error SITE_TITLE env variable not set!");
            return "".to_string();
        }
    }

    let mut config_live_host = "".to_string();
    match std::env::var("ENV_URL") {
        Ok(val) => {
            config_live_host = val.parse().unwrap();
        }
        Err(_) => {
            println!("Error ENV_URL env variable not set!");
            return "".to_string();
        }
    }

    let mut unsubscribe_html = r#"
        You can turn off most email notifications by going to your
        <a href="{config_live_host}/me/settings/">Settings </a>
        then unchecking "Receive Email Notifications" (don't forget to save!). You'll still be able to read your notifications on the site.
    "#.to_string();
    if admin_unsubscribe {
        unsubscribe_html =r#" You've received this message because you've signed up at <a href="{config_live_host}">{config_site_title}</a>.
        This is an administrative message which is not a mailing list."#.to_string()
    }

    unsubscribe_html = unsubscribe_html
        .replace("{config_site_title}", config_site_title.as_ref())
        .replace("{config_live_host}", config_live_host.as_ref())
        .replace("{incoming_message}", incoming_message.as_ref());

    let message = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width" initial-scale="1">
    <!--[if !mso]>
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <![endif]-->
    <meta name="x-apple-disable-message-reformatting">
    <title></title>
    <!--[if mso]>
        <style>
            * { font-family: sans-serif !important; }
        </style>
    <![endif]-->
    <!--[if !mso]><!-->
        <!-- Insert font reference, e.g. <link href="https://fonts.googleapis.com/css?family=Source+Sans+Pro:400,700" rel="stylesheet"> -->
    <!--<![endif]-->
    <style>
        *,
        *:after,
        *:before {
            -webkit-box-sizing: border-box;
            -moz-box-sizing: border-box;
            box-sizing: border-box;
        }
        * {
            -ms-text-size-adjust: 100%;
            -webkit-text-size-adjust: 100%;
        }
        html,
        .documentBG,
        .document {
            width: 100% !important;
            height: 100% !important;
            margin: 0;
            padding: 0;
            background: #eeeeee;
            color: #333;
        }
        body {
            width: 100% !important;
            height: 100% !important;
            margin: 0;
            padding: 0;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
            text-rendering: optimizeLegibility;
            background: #ffffff;
            color: #333;
        }
        div[style*="margin: 16px 0"] {
            margin: 0 !important;
        }
        table,
        td {
            mso-table-lspace: 0pt;
            mso-table-rspace: 0pt;
        }
        table {
            border-spacing: 0;
            border-collapse: collapse;
            table-layout: fixed;
            margin: 0 auto;
        }
        img {
            -ms-interpolation-mode: bicubic;
            max-width: 100%;
            border: 0;
        }
        *[x-apple-data-detectors] {
            color: inherit !important;
            text-decoration: none !important;
        }
        .x-gmail-data-detectors,
        .x-gmail-data-detectors *,
        .aBn {
            border-bottom: 0 !important;
            cursor: default !important;
        }
        .btn {
            -webkit-transition: all 200ms ease;
            transition: all 200ms ease;
        }
        .btn:hover {
            background-color: dodgerblue;
        }
        .mainMessage {
            background: #ffffff;
            text-align: left;
            padding: 15px;
            font-size: 14px;
            color: #333333;
            margin-top: 15px !important;
            margin-botom: 15px !important;
            display: block;
        }

        :not(pre) > code {
            display: block;
            color: #0c0;
            background: #000;
            padding: .5rem;
            border: solid 1px #0c0;
            margin: .25rem;
            white-space:pre-wrap;
            word-wrap:break-word;
        }
        pre > code {
            display: block;
            color: #0c0;
            background: #000;
            padding: .5rem;
            border: solid 1px #0c0;
            margin: .25rem;
            white-space:pre-wrap;
            word-wrap:break-word;
        }

        unsubscribe {
            font-size: 10px;
            text-align: center;
            color: #666666;
            padding: 45px;
            display: block;
        }
        @media screen and (max-width: 750px) {
            .container {
                width: 100%;
                margin: auto;
            }
            .stack {
                display: block;
                width: 100%;
                max-width: 100%;
            }
        }
    </style>
</head>
<body>
    <div class="documentBG">
    <div style="display: none; max-height: 0px; overflow: hidden;">
        <!-- Preheader message here -->
    </div>
    <div style="display: none; max-height: 0px; overflow: hidden;">&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;&zwnj;&nbsp;</div>
    <br />
    <h1 style="text-align: center; font-size: 50px;padding:0 0 0 0;margin: 0 0 0 0;">
        <img src="data:image/png;base64,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" />
        {config_site_title}
    </h1>
<!--    <h2 style="text-align: center; font-size: 20px;padding:0 0 0 0;margin: 0 0 0 0;">
        Email Notification
    </h2>
-->
    <table role="presentation" aria-hidden="true" cellspacing="0" cellpadding="0" border="0" align="center" class="document">
        <tr>
            <td valign="top">
                <table role="presentation" aria-hidden="true" cellspacing="0" cellpadding="0" border="0" align="center" width="750" class="container">
                    <tr>
                        <td>
                            <table role="presentation" aria-hidden="true" cellspacing="0" cellpadding="0" border="0" align="center" width="100%">
                                <tr>
                                    <td class="mainMessage">
                                        {incoming_message}
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                </table>
                <table role="presentation" aria-hidden="true" cellspacing="0" cellpadding="0" border="0" align="center" width="750" class="container">
                    <tr>
                        <td>
                            <!-- Physical address -->
                        </td>
                    </tr>
                    <tr>
                        <td>
                            <!-- Unsubscribe link -->
                            <unsubscribe>
                                {unsubscribe_html}
                            </unsubscribe>
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
    </div>
</body>
</html>
    "#;

    return message
        .replace("{config_site_title}", config_site_title.as_ref())
        .replace("{config_live_host}", config_live_host.as_ref())
        .replace("{incoming_message}", incoming_message.as_ref())
        .replace("{unsubscribe_html}", &unsubscribe_html);
}

pub fn image_to_webp(
    file_path: &String,
    webp_path: &String,
    max_height_or_width: u32,
    crop_square: bool,
) -> Option<String> {
    // Open path as DynamicImage
    let mut image: DynamicImage = ImageReader::open(file_path).unwrap().decode().unwrap();

    if crop_square {
        image = _square_image(image);
    }
    image.thumbnail(max_height_or_width, max_height_or_width);

    // Make webp::Encoder from DynamicImage
    let encoder: Encoder = Encoder::from_image(&image).unwrap();

    // Encode image into WebPMemory
    let encoded_webp: WebPMemory = encoder.encode(65f32);

    // Make File-stream for WebP-result and write bytes into it, and save to path "output.webp"
    let mut webp_image = File::create(&webp_path).unwrap();
    webp_image.write_all(encoded_webp.as_bytes()).unwrap();

    return Some(webp_path.to_owned());
}

pub fn resize_image_max(
    file_path: &String,
    max_height_or_width: u32,
    crop_square: bool,
) -> Option<String> {
    // Open path as DynamicImage
    let mut image: DynamicImage = ImageReader::open(file_path).unwrap().decode().unwrap();

    if crop_square {
        image = _square_image(image);
    }

    image.thumbnail(max_height_or_width, max_height_or_width);

    // Make webp::Encoder from DynamicImage
    let encoder: Encoder = Encoder::from_image(&image).unwrap();

    // Encode image into WebPMemory
    let encoded_webp: WebPMemory = encoder.encode(65f32);

    // Make File-stream for WebP-result and write bytes into it, and save to path "output.webp"
    let mut webp_image = File::create(&file_path).unwrap();
    webp_image.write_all(encoded_webp.as_bytes()).unwrap();

    return Some(file_path.to_owned());
}

fn _square_image(mut image: DynamicImage) -> DynamicImage {
    let (w, h) = image.dimensions();

    if w != h {
        if w > h {
            let new_pos = (w - h) / 2;
            image = image.crop(new_pos, 0, h, h);
        } else {
            let new_pos = (h - w) / 2;
            image = image.crop(0, new_pos, w, w);
        }
    }

    return image;
}

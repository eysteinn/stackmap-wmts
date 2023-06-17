use actix_web::{middleware, web, App, HttpRequest, HttpServer};
use qs::from_str;
//use qstring::QString;
//use std::env;
use std::fs;
use std::io::Write;
use std::process::Output;
use url::Url;
use actix_web::{get, HttpResponse, Responder};
use std::fs::File;
use std::collections::HashMap;

#[macro_use]
//extern crate serde_derive;
extern crate serde_qs as qs;


//async fn index(req: HttpRequest) -> &'static str {
async fn get_capabilities(req: HttpRequest) -> impl Responder { //-> String {
    /*println!("REQ: {req:?}");
    let query_str = req.query_string(); // "name=ferret"
    let qs = QString::from(query_str);
    let name = qs.get("request").unwrap(); // "ferret"
    print!("request: {}", name);*/
    let contents: String = fs::read_to_string("WMTSCapabilities.xml").unwrap();
    //contents
    HttpResponse::Ok()
        .content_type("application/xml")
        .body(contents)
    //"Hello world!"
}

fn imageresponder(filename: String) -> impl Responder {
    let bytes = fs::read("002.png").unwrap();
    //.read_to_end(&mut bytes).unwrap();

    // Set the Content-Type header to indicate that we are returning a PNG image
    HttpResponse::Ok()
        .content_type("image/png")
        .body(bytes)
}

/*pub async fn get_field_value(field: web::Path<(String, String)>) -> HttpResponse {
    field.
}*/


//#[derive(Debug, PartialEq, Deserialize)]
pub struct Request {
    pub request: String,
}

pub struct GetTile {
    pub request: String,
    pub version: String,
    pub layer: String,
    pub style: String,
    pub format: String,
    pub tilematrixset: String,
    pub tilematrix: i32,
    pub tilerow: i32,
    pub tilecol: i32
}

/*async fn getcapabilities(req: HttpRequest) -> String { //impl Responder {
     get_capabilities(req)
}*/
use tile_grid::{Extent, Grid, Unit, Origin};
use std::io::copy;


async fn fetch_image(url: String, tilename: String) -> String{
    //println!("Fetching:  {}", url);

    /*std::fs::File::create(tilename).unwrap(); 
    let response = reqwest::get(url).await;

    let bytes = response.unwrap().bytes() 
    fs::write(tilename, bytes);*/
    //let mut file = std::fs::File::create(&tilename).unwrap();

    let exists = std::path::Path::new(&tilename).exists();
    if exists {
        println!("Using Cached: {}", tilename);
        return tilename
    }
    println!("Downloading from server: {}", tilename);

    let response = reqwest::get(&url).await.unwrap();
    let bytes = response.bytes().await.unwrap();

    fs::write(&tilename, &bytes).unwrap();

    tilename
    /*let response = reqwest::get(url).await;

    let mut dest = File::create(tilename);
    let content = response.text().await;
    copy(&mut content.as_bytes(), &mut dest);*/
}



async fn wmts_service(req: HttpRequest) -> impl Responder {

    let query_str = req.query_string().to_lowercase();
    let map: HashMap<String, String> = qs::from_str(&query_str).unwrap();
    if let Some(value) = map.get("request") {
        if value == "getcapabilities" {
            println!("GetCapabilities");
            let contents: String = fs::read_to_string("WMTSCapabilities.xml").unwrap();
            return HttpResponse::Ok()
            .content_type("application/xml")
            .body(contents)
        }
        if value == "gettile" {
            //println!("GetTile");
            //wmts?request=GetTile&layer=modis-green-snow&time=%7BTime%7D&tilematrixset=webmercator&tilematrix=03&tilecol=1&tilerow=7.png
            

            let mut ytile: u32 = 0;
            let mut xtile: u32 = 0;
            //let mut ytile u32 = 0;
            let mut zoom: u8 = 0;


            if let Some(value) = map.get("tilematrix") {
                let num = value.parse::<u8>(); // parse the string as a u32
                match num {
                    Ok(parsed_zoom) =>  zoom = parsed_zoom,
                    Err(e) => println!("Error parsing number: {}", e),
                } 
            }
            if let Some(value) = map.get("tilecol") {
                let num = value.parse::<u32>(); // parse the string as a u32
                match num {
                    Ok(parsed_value) =>  xtile = parsed_value,
                    Err(e) => println!("Error parsing number: {}", e),
                } 
            }
            if let Some(value) = map.get("tilerow") {
                let num = value.parse::<u32>(); // parse the string as a u32
                match num {
                    Ok(parsed_value) =>  ytile = parsed_value,
                    Err(e) => println!("Error parsing number: {}", e),
                } 
            }
            

            let mut grid = Grid::web_mercator();
            grid.origin = Origin::TopLeft;
            let extent = grid.tile_extent(xtile, ytile, zoom);
            
            //println!("{}, {}, {}, {}", extent.minx, extent.miny, extent.maxx, extent.maxy);
            let tilename = format!("tilecache/{}_{}_{}.png", zoom, xtile, ytile);
            let width = 256;
            let height = 256;
            let url2: String = format!("http://localhost:9080/cgi-bin/mapserv?map=/mapfiles/vedur/rasters.map&program=mapserv&SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX={},{},{},{}&CRS=EPSG:3857&WIDTH={}&HEIGHT={}&LAYERS=modis-green-snow&STYLES=,&CLASSGROUP=black&FORMAT=image/png&TRANSPARENT=true&TIME=2022-03-14T10:40:00Z",
                 extent.minx, extent.miny, extent.maxx, extent.maxy, width, height);
                 //extent.minx, extent.maxy, extent.maxx, extent.miny);
            let tilename = fetch_image(url2, tilename).await;
            //let bytes = fs::read("002.png").unwrap();
            println!("{}, {}", extent.maxx-extent.minx, extent.maxy- extent.miny);
            let bytes = fs::read(tilename).unwrap();
    //.read_to_end(&mut bytes).unwrap();

    // Set the Content-Type header to indicate that we are returning a PNG image
    
            return HttpResponse::Ok()
                .content_type("image/png")
                .body(bytes)
                    //return gettile(req).await
        }
        //return value.to_string()
    }


    HttpResponse::Ok()
    .content_type("text/plain;charset=UTF-8")
    .body("Request parameter missing or not of right type")
    //sString::from("done")
}

async fn gettile(req: HttpRequest) -> impl Responder {
    let query_str = req.query_string().to_lowercase();
    //let input: Request = qs::from_str(&query_str).unwrap();
    let map: HashMap<String, String> = qs::from_str(&query_str).unwrap();
    
    if let Some(value) = map.get("foo") {

        println!("Found FOO!");
        println!("{}", value);
        let contents: String = fs::read_to_string("WMTSCapabilities.xml").unwrap();
        //contents
        return HttpResponse::Ok()
            .content_type("application/xml")
            .body(contents)
    } 

    let bytes = fs::read("002.png").unwrap();
    //.read_to_end(&mut bytes).unwrap();

    // Set the Content-Type header to indicate that we are returning a PNG image
    
    HttpResponse::Ok()
        .content_type("image/png")
        .body(bytes)
    //println!("{:?}", map);
    //"DONE"
    /*if input.request == "gettile" {
        println!("Processing GetTile");
        imageresponder("abc".to_string())
    } else {*/
        /*println!("input: {}", input.request);

        format!("Request: {}", input.request)*/

        //imageresponder("abc".to_string())
    //}


    /*let url = Url::try_from(req.query_string()).unwrap();

    Url::parse(input)

    //let qs = QString::from(query_str);

    url.query().
    
    if !qs.has("request") {
        return "Missing Request parameter";
    }

    let request = qs.get("request").unwrap().to_lowercase();

    if request == "getcapabilities" {
        "Request is GetCapabilities"
    } else if request == "gettile" {
        "Request is GetTile"
    } else {
        "Error"
    }*/
   // "Hallo"
}

async fn tile(req: HttpRequest) -> impl Responder {
    //let mut file = File::open("image.png").unwrap();
    print!("path: {}", req.path());
    /*let query_str = req.query_string(); // "name=ferret"
    let qs = QString::from(query_str);

    if qs.get(name).contains("request") {
        let qs = QString::from(query_str);
        print!("{}",qs);
        imageresponder(qs.to_string())
    } else {
        "Nothing"
    }*/

    "Nothing"
    /*let mut bytes = Vec::new();
    bytes = fs::read("002.png").unwrap();
    //.read_to_end(&mut bytes).unwrap();

    // Set the Content-Type header to indicate that we are returning a PNG image
    HttpResponse::Ok()
        .content_type("image/png")
        .body(bytes)*/

    /*Ok(HttpResponse::build(StatusCode::OK)
        .content_type("image/jpeg")
        .body(image_content))*/

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //log::info!("starting HTTP server at http://localhost:9099");
    println!("starting HTTP server at http://localhost:9099");

    HttpServer::new(|| {
        App::new()
            // enable logger
            //.wrap(middleware::Logger::default())
            //.service(web::resource("/wmts/1.0.0/getcapabilities").route(web::get().to(get_capabilities)))
            .service(web::resource("/wmts/1.0.0/GetCapabilities").route(web::get().to(get_capabilities)))
            .service(web::resource("/wmts/1.0.0/GetTile").to(gettile))
            .service(web::resource("/wmts").to(wmts_service))
            //.service(web::resource("/wmts/{tail}*").route(web::get().to(tile)))
            //.service(web::resource("/index.html").route(web::get().to(|| async { "Hello world!" }))
            //.service(web::resource("/").to(index))
    })
    .bind(("127.0.0.1", 9099))?
    .run()
    .await
}


//http://localhost:9080/cgi-bin/mapserv?map=/mapfiles/vedur/rasters.map&program=mapserv&SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=60.63023877468341,-38.04788694962596,85.41639837197434,72.08181802540517&CRS=EPSG:4326&WIDTH=1024&HEIGHT=768&LAYERS=modis-green-snow&STYLES=,&CLASSGROUP=black&FORMAT=image/png&TRANSPARENT=true&TIME=2022-03-14T10:40:00Z
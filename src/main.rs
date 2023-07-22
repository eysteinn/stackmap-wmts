use actix_web::{web, App, HttpRequest, HttpServer};
use std::env;
use std::fs;
use actix_web::{HttpResponse, Responder};
use std::collections::HashMap;
use chrono::{DateTime};

#[macro_use]
//extern crate serde_derive;
extern crate serde_qs as qs;

//static WMTSCapabilitiesFile :&str = include_str!("../WMTSCapabilities.xml");


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
    let path = std::path::Path::new(&tilename);
    
    //let exists = std::path::Path::new(&tilename).exists();
    
    if path.exists() {
        println!("Using Cached: {}", tilename);
        return tilename
    }
    println!("Downloading from server: {}", tilename);
    //println!("Using url: {}", url);


    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create parent directory");
    }

    let response: reqwest::Response = reqwest::get(&url).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    
    //fs::write(&tilename, &bytes).unwrap();
    fs::write(path, &bytes).unwrap();

    tilename
    /*let response = reqwest::get(url).await;

    let mut dest = File::create(tilename);
    let content = response.text().await;
    copy(&mut content.as_bytes(), &mut dest);*/
}


//async fn leaflet_service(route: web::Path<(String, String,)>, req: HttpRequest) -> impl Responder {
async fn leaflet_service_dynamic(route: web::Path<(String, String,)>, req: HttpRequest) -> HttpResponse {
    let (project, layer) = route.into_inner();
    leaflet_service(project, layer, req).await
}

async fn leaflet_service_query(query_params: web::Query<HashMap<String, String>>, // Deserialize query parameters into HashMap
    req: HttpRequest) -> HttpResponse {
    
    let project = query_params.get("project").cloned().unwrap_or_default();

    let layer = query_params.get("layer").cloned().unwrap_or_default();
    
    if !layer.is_empty() && !project.is_empty() {
        return leaflet_service(project, layer, req).await;
    }
    return HttpResponse::BadRequest()
    .content_type("text/plain;charset=UTF-8")
    .body("Request parameters 'project' or 'layer' missing or not of the right type")
    
}

async fn leaflet_service(project: String, layer: String, req: HttpRequest) -> HttpResponse {
    //let (project, layer) = route.into_inner();
    /*let wmts_domain: String;
    if let Ok(value) = env::var("WMTS_DOMAIN") {
        wmts_domain = value;
        println!("Found WMTS_DOMAIN env variable")
    } else {
        let scheme = req.connection_info().scheme().to_owned();
        let host = req.connection_info().host().to_owned();
        wmts_domain = format!("{}://{}", scheme, host);
    }*/
    let wmts_domain = get_domain(req);
    let mut leaflet = include_str!("leaflet_embed.html").to_string();
    leaflet = leaflet.replace("{LAYER}", &layer).replace("{PROJECT}", &project).replace("{WMTS_DOMAIN}", &wmts_domain);
    /*let path: String = ""
    let mut contents: String = fs::read_to_string(path).unwrap();*/
    

    return HttpResponse::Ok()
    .content_type("text/html")
    .body(leaflet)
}

async fn wmts_service_dynamic(route: web::Path<(String,)>, req: HttpRequest) -> impl Responder {
    let project: String = route.into_inner().0.to_lowercase();
    wmts_service(project, req).await
}


async fn wmts_service_query(query_params: web::Query<HashMap<String, String>>, req: HttpRequest) -> HttpResponse {
    /*let query_str = req.query_string().to_lowercase();
    let query_map: HashMap<String, String> = qs::from_str(&query_str).unwrap();*/
    
    if let Some(value) = query_params.get("project") {
        return wmts_service(value.to_string(), req).await;
    }
    
    HttpResponse::BadRequest()
        .content_type("text/plain;charset=UTF-8")
        .body("Request parameter 'project' missing or not of the right type")
}

async fn wmts_service_query2(req: HttpRequest) -> impl Responder {
    let query_str = req.query_string().to_lowercase();
    let query_map: HashMap<String, String> = qs::from_str(&query_str).unwrap();
    if let Some(value) = query_map.get("project") {
        return wmts_service(value.to_string(), req).await;
    }
    HttpResponse::Ok()
    .content_type("text/plain;charset=UTF-8")
    .body("Request parameter missing or not of right type")
}
fn get_domain(req: HttpRequest) -> String {
    let wmts_domain: String;
    if let Ok(value) = env::var("WMTS_DOMAIN") {
        wmts_domain = value;
        println!("Found WMTS_DOMAIN env variable")
    } else {
        let scheme = req.connection_info().scheme().to_owned();
        let host = req.connection_info().host().to_owned();
        wmts_domain = format!("{}://{}", scheme, host);
    }
    wmts_domain
}
async fn wmts_service(project: String, req: HttpRequest) -> HttpResponse { // impl Responder {

    /*let domain = req.connection_info().host().to_owned();
    println!("Domain: {}", domain);
    println!("Scheme: {}", req.connection_info().scheme());
    println!("peer_addr: {}", req.connection_info().peer_addr().unwrap());
    let host = req.headers().get("Host").unwrap();
    println!("Host: {}", host.to_str().unwrap());*/
    /*let wmts_host: String;
    if let Ok(value) = env::var("WMTS_HOST") {
        wmts_host = value;
    } else 
        wmts_host = "http://localhost:9099".to_string();
        println!("WMTS_HOST not found, setting value to {}", wmts_host);*/
    //let wmts_host = env::var("EXTERNAL_WMTS_HOST").expect("Environmental variable EXTERNAL_WMTS_HOST not found");
    //let project: String = route.into_inner().0.to_lowercase();
    // TODO: need to make sure that 'project' string does not contain any '../' to prevent injection

    let query_str = req.query_string().to_lowercase();
    let query_map: HashMap<String, String> = qs::from_str(&query_str).unwrap();
    if let Some(value) = query_map.get("request") {
        
        if value == "getcapabilities" {
            println!("GetCapabilities");
            /*let wmts_domain: String;
            if let Ok(value) = env::var("WMTS_DOMAIN") {
                wmts_domain = value;
                println!("Found WMTS_DOMAIN env variable")
            } else {
                let scheme = req.connection_info().scheme().to_owned();
                let host = req.connection_info().host().to_owned();
                wmts_domain = format!("{}://{}", scheme, host);
            }*/
            let wmts_domain = get_domain(req);
            let path = format!("./projects/{}/WMTSCapabilities.xml", project);
            let mut contents: String = fs::read_to_string(path).unwrap();
            contents = contents.replace("{WMTS_DOMAIN}", &wmts_domain);
            
            //contents = contents.replace("{EXTERNAL_WMTS_HOST}", &wmts_host).replace("{PROJECT}", &project);
            //contents = contents.replace("{PROJECT}", &project);
            return HttpResponse::Ok()
            .content_type("application/xml")
            .body(contents)
        }
        if value == "gettile" {
            //println!("GetTile");
            //wmts?request=GetTile&layer=modis-green-snow&time=%7BTime%7D&tilematrixset=webmercator&tilematrix=03&tilecol=1&tilerow=7.png
            //let wms_host = env::var("WMS_HOST").expect("Environmental variable WMS_HOST not found");

            let wms_host: String;
            if let Ok(value) = env::var("WMS_HOST") {
                wms_host = value;
            } else {
                wms_host = "stackmap-mapserver.default.svc.cluster.local".to_string();
                println!("Environmental variable WMS_HOST not found, setting to {}", wms_host);
            }
            
            let zoom = query_map.get("tilematrix").unwrap().parse::<u8>().unwrap();
            let xtile = query_map.get("tilecol").unwrap().parse::<u32>().unwrap();
            let ytile = query_map.get("tilerow").unwrap().parse::<u32>().unwrap();
            let layer = query_map.get("layer").expect("Missing parameter: LAYER");
            let mut timestr: String = query_map.get("time").cloned().unwrap_or("".to_string());
            //let mut dt : DateTime<chrono::UTC>;
            let mut timedir: String = "default".to_string();
            if !timestr.is_empty() {
                /*Use Chrono to format the timestr in to actual datetime object. 
                This will make directory structure predictable.*/
                //let dt = timestr.parse::<DateTime<chrono::UTC>>().expect("Could not parse time parameter");
                let dt = timestr.parse::<DateTime<chrono::UTC>>().expect("Could not parse time parameter");
                timestr = dt.format("%Y-%m-%dT%H:%M:%S").to_string();
                timedir = dt.format("%Y%m%dT%H%M%S").to_string();
            }
            //let timestr = query_map.get("time").unwrap_or("").to_string();

            let mut grid = Grid::web_mercator();
            grid.origin = Origin::TopLeft;
            let extent = grid.tile_extent(xtile, ytile, zoom);
            
            //println!("{}, {}, {}, {}", extent.minx, extent.miny, extent.maxx, extent.maxy);
            
            let tilename = format!("tilecache/{}/{}/{}/{}_{}_{}.png", project, layer, timedir, zoom, xtile, ytile);
            let width = 256;
            let height: i32 = 256;
            /*let url: String = format!("http://localhost:9080/cgi-bin/mapserv?map=/mapfiles/vedur/rasters.map&program=mapserv&SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX={},{},{},{}&CRS=EPSG:3857&WIDTH={}&HEIGHT={}&LAYERS=modis-green-snow&STYLES=,&CLASSGROUP=black&FORMAT=image/png&TRANSPARENT=true&TIME=2022-03-14T10:40:00Z",
                 extent.minx, extent.miny, extent.maxx, extent.maxy, width, height);*/
            

            let mut url: String = format!("{}?map=/mapfiles/{}/rasters.map&SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX={},{},{},{}&CRS=EPSG:3857&WIDTH={}&HEIGHT={}&LAYERS={}&STYLES=,&CLASSGROUP=black&FORMAT=image/png&TRANSPARENT=true", //&TIME=2022-03-14T10:40:00Z",
            wms_host, project, extent.minx, extent.miny, extent.maxx, extent.maxy, width, height, layer);
                 //extent.minx, extent.maxy, extent.maxx, extent.miny);
            
            if !timestr.is_empty() {
                url = format!("{}&time={}", url, timestr);
            }
            println!("{}", url);

            let tilename = fetch_image(url, tilename).await;
            //let bytes = fs::read("002.png").unwrap();
            //println!("{}, {}", extent.maxx-extent.minx, extent.maxy- extent.miny);
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

async fn default_route(req: HttpRequest) -> HttpResponse {
    println!("Default route: {}", req.uri());
    HttpResponse::NotFound()
        .content_type("text/plain;charset=UTF-8")
        .body("Project not found")
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
            //.service(web::resource("/wmts/1.0.0/GetCapabilities").route(web::get().to(get_capabilities)))
            //.service(web::resource("/wmts/1.0.0/GetTile").to(gettile))
            .service(web::resource("/projects/{project}/services/wmts").to(wmts_service_dynamic))
            .service(web::resource("/projects/{project}/layers/{layer}/leaflet").to(leaflet_service_dynamic))
            
            .service(web::resource("/services/wmts").to(wmts_service_query))
            .service(web::resource("/services/leaflet").to(leaflet_service_query))

            //.service(web::resource("/wmts/{tail}*").route(web::get().to(tile)))
            //.service(web::resource("/index.html").route(web::get().to(|| async { "Hello world!" }))
            //.service(web::resource("/").to(index))


            //In Default service, show what is the url!!! This will help debug qgis
            //.default_service(web::route().to(|| {println!("Default Route"); HttpResponse::NotFound() }))
            .default_service(web::route().to(default_route))
    })
    .bind(("127.0.0.1", 9099))?
    .run()
    .await
    /*let app = App::new()
    .service(web::resource("/wmts").to(wmts_service))
    .default_service(web::to(|| HttpResponse::NotFound()));*/
    
}


//http://localhost:9080/cgi-bin/mapserv?map=/mapfiles/vedur/rasters.map&program=mapserv&SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=60.63023877468341,-38.04788694962596,85.41639837197434,72.08181802540517&CRS=EPSG:4326&WIDTH=1024&HEIGHT=768&LAYERS=modis-green-snow&STYLES=,&CLASSGROUP=black&FORMAT=image/png&TRANSPARENT=true&TIME=2022-03-14T10:40:00Z

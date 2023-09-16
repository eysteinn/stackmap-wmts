use std::{ops::Bound, str::FromStr};

use chrono::{DateTime};
use tile_grid::Grid;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use handlebars::Handlebars;


const XML_LAYER_TEMPLATE: &str = include_str!("layer.xml");

#[derive(Serialize, Deserialize)]
pub struct ServiceIdentification {
    pub title: String,
    pub service_abstract: String,
    pub service_type: String,
    pub service_type_version: String,
    pub fees: String,
    pub access_constraints: String,
}


impl Default for ServiceIdentification {
    fn default() -> Self {
        ServiceIdentification {
            title: "Stackmap WMTS server".to_string(),
            service_abstract: "Generic WMTS capability report.".to_string(),
            service_type: "OGC WMTS".to_string(),
            service_type_version: "1.0.0".to_string(),
            fees: "none".to_string(),
            access_constraints: "none".to_string()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LayerDefinition {
    //pub project: String,
    pub product: String,
    pub title: String,
    pub description: String,
    //pub enable_wmts: bool,
    pub format: String,
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
    /*pub tilematrixset: Vec<String>,
    pub resource_link: String,*/

    //pub times:  Vec<DateTime<chrono::UTC>>,
}


impl Default for LayerDefinition {
    fn default() -> Self {
        LayerDefinition {
            product: "".to_string(),
            title: "".to_string(),
            description: "".to_string(),
            format: "image/png".to_string(),
            min_lat: -85.0511287798066,
            max_lat: 85.0511287798066,
            min_lon: -180.0,
            max_lon: 180.0
        }
    }
}

#[derive(Serialize)]
struct WMTSDocument<'a> {
    service_id: &'a ServiceIdentification,
    layers: &'a Vec<LayerDefinition>,
    project: &'a String,
    domain: &'a String,
}

pub fn layer_to_xml(layerdef: LayerDefinition) -> String {
    let handlebars = Handlebars::new();
    let mut context = handlebars::to_json(layerdef);
    context.as_object_mut().unwrap().insert("user".to_string(), handlebars::to_json("Eysteinn"));
    handlebars.render_template(XML_LAYER_TEMPLATE, &context).unwrap()
}

#[derive(Debug, Deserialize)]
struct UniqueProducts {
    product_names: Vec<String>
}

pub async fn get_products_from_api(host: &str, project: &str) -> Vec<String> {

    let mut url = "{host}/api/v1/projects/{project}/products".to_string();
    url = url.replace("{project}", project).replace("{host}", host);

    println!("URL: {}", url);

    let client = reqwest::Client::new();
    let response = client.get(url).send().await.unwrap();
    //let response2 = reqwest::blocking::get(url).expect("Could not fetch url");
    println!("Got a response");
    
    let var: UniqueProducts = response.json().await.unwrap(); //.unwrap();

    
    println!("{:?}", var.product_names);
    
    var.product_names
}
/* 
pub fn xml_test() -> String {
    let project = "vedur";
    //let host = "http://stackmap.clouds.is:8080";
    let host = std::env::var("WMTS_HOSTNAME").unwrap_or("http://stackmap.clouds.is:8080".to_string());

    let handlebars = Handlebars::new();

    let si = ServiceIdentification::default();
   
    let mut layers: Vec<LayerDefinition> = Vec::new();
    for product in get_products_from_api(&host, project) {
        let layer = LayerDefinition{
            product: product.to_string(),
            //product: String::from_str("viirs-granule-true-color"),
            title: product.to_string(),
            description: "".to_string(),
            ..Default::default()
        };
        layers.push(layer)
    }

    //let mut context = handlebars::new(); //:to_json(ld);
    let combined_context: WMTSDocument<'_> = WMTSDocument {
        service_id: &si,
        layers: &layers,
        project: &project.to_string(),
        domain: &host,
    };

    handlebars.render_template(XML_LAYER_TEMPLATE, &combined_context).unwrap()

    //handlebars.render_template(xml_layer, &ld).unwrap()
}*/
impl LayerDefinition {
    
    pub fn to_xml(&self) -> String {
        let mut context = handlebars::to_json(self);
        context.as_object_mut().unwrap().insert("user".to_string(), handlebars::to_json("Eysteinn"));

        let handlebars = Handlebars::new();
        handlebars.render_template(XML_LAYER_TEMPLATE, &context).unwrap()
        
    }
}


pub async fn make_xml(project: String, host: String, apihost: String) -> String {
    let handlebars = Handlebars::new();

    let si = ServiceIdentification::default();
    
    let mut layers: Vec<LayerDefinition> = Vec::new();
    for product in get_products_from_api(&apihost, &project).await {
        let layer = LayerDefinition{
            product: product.to_string(),
            //product: String::from_str("viirs-granule-true-color"),
            title: product.to_string(),
            description: "".to_string(),
            ..Default::default()
        };
        layers.push(layer)
    }

    let combined_context: WMTSDocument<'_> = WMTSDocument {
        service_id: &si,
        layers: &layers,
        project: &project.to_string(),
        domain: &host,
    };

    handlebars.render_template(XML_LAYER_TEMPLATE, &combined_context).unwrap()

}
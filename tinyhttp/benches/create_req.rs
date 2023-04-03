use std::rc::Rc;
use criterion::{criterion_group, criterion_main, Criterion};


use std::collections::HashMap;

/// Struct containing data on a single request.
///
/// parsed_body which is a Option<String> that can contain the body as a String
///
/// body is used when the body of the request is not a String

#[derive(Clone)]
pub struct Request {
    parsed_body: Option<String>,
    headers: HashMap<String, String>,
    status_line: Vec<String>,
    body: Vec<u8>,
    wildcard: Option<String>,
}

#[derive(Clone, Debug)]
pub enum BodyType {
    ASCII(String),
    Bytes(Vec<u8>),
}

impl Request {
    pub(crate) fn new(
        raw_body: Vec<u8>,
        raw_headers: Vec<String>,
        status_line: Vec<String>,
        wildcard: Option<String>,
    ) -> Request {
        let raw_body_clone = raw_body.clone();
        let ascii_body = match std::str::from_utf8(&raw_body_clone) {
            Ok(s) => Some(s),
            Err(_) => {
                #[cfg(feature = "log")]
                log::info!("Not an ASCII body");
                None
            }
        };

        let mut headers: HashMap<String, String> = HashMap::new();
        #[cfg(feature = "log")]
        log::trace!("Headers: {:#?}", raw_headers);
        for i in raw_headers.iter() {
            let mut iter = i.split(": ");
            let key = iter.next().unwrap();
            let value = iter.next().unwrap();

            /*            match value {
                            Some(v) => println!("{}", v),
                            None => {
                                break;
                            }
                        };
            */
            headers.insert(key.to_string(), value.to_string());
        }

        #[cfg(feature = "log")]
        log::info!("Request headers: {:?}", headers);

        if ascii_body.is_none() {
            Request {
                parsed_body: None,
                body: raw_body,
                headers,
                status_line,
                wildcard,
            }
        } else {
            Request {
                body: raw_body,
                parsed_body: Some(ascii_body.unwrap().to_string()),
                headers,
                status_line,
                wildcard,
            }
        }
    }

    pub(crate) fn set_wildcard(mut self, w: Option<String>) -> Self {
        self.wildcard = w;
        self
    }

    pub fn get_raw_body(&self) -> Vec<u8> {
        self.body.clone()
    }

    pub fn get_parsed_body(&self) -> Option<String> {
        self.parsed_body.clone()
    }

    pub fn get_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    pub fn get_status_line(&self) -> Vec<String> {
        self.status_line.clone()
    }

    pub fn get_wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
}


fn build_and_parse_req(buf: Vec<u8>) -> Request {
    let mut safe_http_index = buf.windows(2).enumerate();
    let status_line_index_opt = safe_http_index
        .find(|(_, w)| matches!(*w, b"\r\n"))
        .map(|(i, _)| i);

    let status_line_index = if status_line_index_opt.is_some() {
        status_line_index_opt.unwrap()
    } else {
        #[cfg(feature = "log")]
        log::info!("failed parsing status line!");

        0usize
    };

    let first_header_index = safe_http_index
        .find(|(_, w)| matches!(*w, b"\r\n"))
        .map(|(i, _)| i)
        .unwrap();

    #[cfg(feature = "log")]
    log::debug!(
        "STATUS LINE: {:#?}",
        std::str::from_utf8(&buf[..status_line_index])
    );

    #[cfg(feature = "log")]
    log::debug!(
        "FIRST HEADER: {:#?}",
        std::str::from_utf8(&buf[status_line_index + 2..first_header_index])
    );

    let mut headers = Vec::<String>::new();
    let mut headers_index = vec![first_header_index + 2];
    loop {
        
        let header_index: usize = match safe_http_index
            .find(|(_, w)| matches!(*w, b"\r\n"))
            .map(|(i, _)| i + 2)
        {
            Some(s) => s,
            _ => break,
        };

        #[cfg(feature = "log")]
        log::trace!("header index: {}", header_index);

        let header =
            String::from_utf8(buf[*headers_index.last().unwrap()..header_index - 2].to_vec())
                .unwrap()
                .to_lowercase();
        if header.is_empty() {
            break;
        }
        #[cfg(feature = "log")]
        log::trace!("HEADER: {:?}", headers);

        headers_index.push(header_index);
        headers.push(header);
    }

    let iter_status_line = String::from_utf8(buf[..status_line_index].to_vec()).unwrap();

    //let headers = parse_headers(http.to_string());
    let str_status_line = Vec::from_iter(iter_status_line.split_whitespace());
    let status_line: Rc<Vec<String>> =
        Rc::new(str_status_line.iter().map(|i| String::from(*i)).collect());
    #[cfg(feature = "log")]
    log::debug!("{:#?}", status_line);
    let body_index = buf
        .windows(4)
        .enumerate()
        .find(|(_, w)| matches!(*w, b"\r\n\r\n"))
        .map(|(i, _)| i)
        .unwrap();

    let raw_body = &buf[body_index + 4..];
    #[cfg(feature = "log")]
    log::debug!(
        "BODY (TOP): {:#?}",
        std::str::from_utf8(&buf[body_index + 4..]).unwrap()
    );
    Request::new(
        raw_body.to_vec(),
        headers.clone(),
        status_line.to_vec(),
        None,
    )
}


pub fn criterion_benchmark(c: &mut Criterion) {
    let http = b"POST /hello_world HTTP/1.1\r\n
                        Accept-Encoding: gzip\r\n
                        Accept-Content: text/plain\r\n\r\n
                        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Ut egestas nunc neque, ac condimentum diam commodo vitae. Morbi ac orci eget justo cursus sodales vel at ipsum. Pellentesque massa diam, congue a elit sit amet, elementum sagittis tellus. Nulla facilisi. Vestibulum dapibus sem quis sollicitudin rhoncus. Nam eleifend purus eget nisi tristique, eget pellentesque felis porta. Ut viverra mattis pulvinar. Vestibulum velit quam, blandit vel feugiat gravida, bibendum ut nibh. Morbi euismod ultricies luctus. Nam sodales feugiat lacus. Duis faucibus turpis nisi, in feugiat tortor bibendum id. Integer dignissim, arcu ut luctus semper, ante neque laoreet urna, eleifend ultrices quam erat a nibh. Sed et porta magna, quis cursus turpis.

Duis et diam eleifend, tristique tellus efficitur, bibendum dui. Etiam porttitor a mauris et sagittis. Suspendisse potenti. Proin varius augue a dui feugiat volutpat. Sed sollicitudin nibh ipsum, nec gravida nisl sodales ut. Nulla vel faucibus nulla, in lobortis lacus. Proin sapien mi, cursus in tellus mollis, finibus consequat ex. Aenean non tellus elit. Ut gravida nisl eu turpis convallis aliquam vitae ac urna. Proin congue augue ac elementum congue. In vestibulum cursus libero vel vehicula. Curabitur vel aliquam eros. Aenean a turpis ullamcorper, viverra sem sed, pellentesque purus.

Pellentesque pellentesque purus ante, a tempor odio porta sed. Praesent dictum consequat dui id posuere. Vivamus in orci eget odio tempor condimentum. Pellentesque feugiat est in nunc tristique rutrum. Morbi cursus nisi eu nisl fermentum, eget gravida urna sollicitudin. Donec a leo vel ligula posuere feugiat. Integer libero risus, consequat eu enim et, suscipit maximus diam. Donec sit amet sagittis erat. Morbi vehicula in mauris et euismod. Vestibulum aliquet tristique mattis. Duis maximus viverra risus, ullamcorper tincidunt dolor ornare id. Morbi blandit, massa quis aliquam mollis, neque ligula blandit mi, ac iaculis justo erat quis nisl. Nam fermentum ullamcorper est. Morbi ullamcorper egestas dolor, semper blandit nibh aliquet et. Vivamus ligula tellus, ultricies sed orci at, fringilla ultricies risus. Nullam sagittis, felis vel placerat sollicitudin, lectus risus condimentum nisi, in tincidunt dui tortor in nisl.

Suspendisse potenti. Fusce dapibus vestibulum aliquam. Integer nec lectus ex. Maecenas varius consectetur mi, eu aliquet libero. Duis placerat ligula vitae justo sodales, a cursus tortor placerat. Curabitur porttitor, elit gravida interdum scelerisque, felis enim placerat nisi, eget elementum magna odio non nunc. Maecenas scelerisque dui ac lorem efficitur interdum. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam dignissim, est nec feugiat posuere, lectus diam convallis mauris, in fringilla sapien nibh nec orci. Sed eu mattis elit, sit amet vulputate nisl.

Maecenas at urna et arcu eleifend feugiat. Quisque suscipit dolor quis nulla aliquam sollicitudin. Proin aliquam consequat tortor, sed gravida nibh auctor nec. Nulla non metus sapien. Maecenas vel enim sed ligula sollicitudin bibendum. Etiam eget rhoncus nunc. In cursus eros urna, vitae euismod urna ultricies in. Mauris euismod elementum sapien, ut euismod odio.

Aenean ultricies elit dui, in ornare mi consectetur non. Ut quis est felis. In nec finibus metus. Nam vel sem nunc. Cras convallis diam eget elementum tincidunt. Phasellus eu sapien eu leo tincidunt ultrices in ut ante. Donec eu nibh quis odio blandit rhoncus. Maecenas venenatis augue quis nibh volutpat varius. Fusce id porttitor arcu, vitae dapibus felis. Donec semper neque at rhoncus dapibus.

Donec ornare sagittis nulla, ut tempor mi lobortis ut. Praesent vitae metus id ante aliquet sodales. Quisque sollicitudin varius tincidunt. In quis porttitor massa. Aenean in neque at ipsum efficitur fringilla. Phasellus at libero turpis. Sed quis facilisis mi. Integer nec leo auctor, molestie velit non, vestibulum neque. Etiam posuere eros magna, ac tincidunt nibh pharetra vitae. Vestibulum malesuada lectus turpis, cursus feugiat odio pretium elementum. Nullam pellentesque massa non placerat porttitor. Praesent scelerisque, mauris ut finibus malesuada, elit ligula cursus justo, sit amet dapibus erat ante in arcu. Nam dignissim metus sed lorem hendrerit, in tempus tellus luctus. Pellentesque finibus, erat ac tempus varius, leo erat cursus diam, at vulputate ipsum diam quis neque.

Etiam at ultrices nunc. Aliquam imperdiet est in arcu feugiat scelerisque. Quisque sit amet eleifend libero, a dapibus sapien. Ut laoreet eu sapien vitae tincidunt. Mauris dictum eleifend lacus et laoreet. Cras tempus mollis elit, non pretium justo porta sed. Praesent a ante in lacus tempus vestibulum. Mauris ac ipsum velit. Nulla venenatis orci sed sem aliquet, vel interdum ipsum tristique. Nulla molestie ultrices ante, ac convallis arcu euismod in. Sed at velit quis ex finibus dictum.

Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Ut at fermentum mi. Pellentesque imperdiet ultricies libero vitae malesuada. Nulla sit amet iaculis ante, in condimentum sem. Proin ac ex eget lectus pretium euismod. Ut sapien odio, semper eget volutpat ac, vestibulum id nisi. Fusce feugiat, nibh nec ornare sagittis, lectus tortor accumsan leo, at mollis erat elit sit amet ipsum. Cras eget lacus in ex tristique maximus a ullamcorper augue. Quisque nec semper est. Proin sed eros sed velit dignissim sollicitudin. Nunc fringilla hendrerit rhoncus. Suspendisse dictum rhoncus convallis. Mauris sagittis ipsum ac placerat sollicitudin. Morbi eleifend erat nec aliquam euismod. Donec vulputate velit ac tortor ullamcorper, at placerat risus ultrices. Cras molestie, ligula vitae pellentesque consectetur, tortor nunc facilisis mauris, ut volutpat risus massa nec tortor.

Donec fermentum ante eget tristique bibendum. Nunc eu sollicitudin lorem, ac cursus erat. Morbi nunc nisl, dapibus id iaculis vitae, maximus non dui. Aliquam ultrices purus elit, a suscipit ipsum ultricies eu. Suspendisse eros orci, lobortis id justo sit amet, convallis rutrum orci. Ut congue augue pellentesque mi ornare suscipit. Quisque nec arcu finibus, fringilla nisi nec, pellentesque dui. Fusce ac dignissim neque. Integer non lorem risus. Fusce sollicitudin accumsan ligula, vel accumsan risus placerat vitae. Nullam ac aliquet erat. Vivamus ornare tincidunt purus, vitae lacinia neque condimentum id. Nullam ac neque ac neque sodales finibus eu quis massa. Proin tristique, purus a tincidunt tincidunt, sapien ligula gravida risus, at tempor enim nisl ac felis. Duis in luctus massa. Sed ut laoreet ipsum.

Suspendisse eu aliquet justo. Nullam vehicula imperdiet ipsum sed lacinia. Integer non elit semper, euismod sem non, ullamcorper justo. Pellentesque feugiat diam lorem, sit amet rutrum ipsum ornare eu. Donec ut auctor augue. Phasellus ac tempus diam, vitae blandit mi. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Phasellus mi turpis, aliquet sit amet volutpat bibendum, porttitor sed augue. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Sed suscipit cursus ex, nec ullamcorper nulla feugiat non. Proin ultrices porttitor ante in commodo. Phasellus pulvinar, urna sagittis dictum ullamcorper, nisl purus sollicitudin quam, in varius ipsum justo ut risus. Maecenas nec consequat urna, quis vehicula magna. Quisque posuere lacus vel scelerisque efficitur. Nam mattis augue vitae metus ornare semper sed at eros.

Nam et felis auctor, convallis lacus in, luctus mi. Nunc posuere dictum nisi. Quisque sapien odio, aliquet et egestas eu, fringilla non massa. Nam ac varius purus, sit amet vulputate purus. Nulla facilisi. Integer commodo ligula turpis, vitae luctus neque sollicitudin in. Maecenas molestie dolor felis, sed pulvinar nulla tempus nec. Interdum et malesuada fames ac ante ipsum primis in faucibus. Vivamus vehicula nisi erat, eget lobortis ex sodales in. Vestibulum laoreet sodales enim eget eleifend. Nulla ornare iaculis lorem sed consectetur. Nullam dapibus, tellus a congue lacinia, purus nunc vehicula ipsum, nec faucibus diam turpis eget purus. Curabitur feugiat ex sapien, at luctus metus faucibus iaculis.

Donec ultricies sapien neque, vel hendrerit ligula sagittis in. Morbi at risus dictum, tempus mauris at, sollicitudin est. Cras consectetur fringilla lacus, quis congue justo mollis quis. Vestibulum sodales et purus sed convallis. In malesuada nibh risus, volutpat rutrum neque consequat sodales. Proin sem ex, mattis at magna vitae, lobortis tincidunt mi. Quisque quis enim eget lacus eleifend dignissim eu sit amet odio. Suspendisse malesuada pulvinar tellus a dictum. Duis in metus at risus gravida suscipit sit amet quis erat. Maecenas sit amet eros eros. Nulla accumsan, dolor nec fringilla commodo, nisl est suscipit est, et hendrerit nibh risus ut felis. Vestibulum odio nunc, commodo non justo a, interdum venenatis ligula. Aenean sodales, est dignissim dictum porttitor, lacus dolor pulvinar lorem, vel ultrices leo arcu vitae risus. Ut in velit eget nunc pharetra molestie a nec tellus.

Praesent at orci magna. Nam vitae suscipit mauris, vitae tempor erat. Nullam vitae odio vitae risus sagittis pharetra. Curabitur augue urna, congue at nisl quis, cursus facilisis tortor. Etiam maximus mattis quam. Quisque sit amet ultrices orci. Praesent non sem nibh. Ut at tincidunt tellus. Quisque at bibendum mi.

Etiam vehicula nunc orci, nec imperdiet mi dignissim id. Ut tempus eros arcu. Curabitur non erat et leo ornare maximus eu id eros. Curabitur eu ante ac neque porta cursus. Praesent vel neque leo. Aenean est nunc, tincidunt eget congue eget, porta pretium nisl. Fusce nisi orci, tempus eu est ac, molestie lacinia lacus. Cras leo nibh, dignissim vitae vestibulum vitae, tincidunt ut libero. Aliquam erat volutpat. Suspendisse potenti. Donec tincidunt, est ac mollis sagittis, mauris tortor semper tortor, suscipit dapibus sapien nibh in sem.

Fusce in pellentesque metus. Maecenas sed vulputate lectus, ornare vestibulum est. Duis blandit eget ipsum vitae blandit. Maecenas eget eleifend leo. Nam orci eros, porttitor sit amet pellentesque a, suscipit a sem. Donec rutrum tellus tempor leo porttitor luctus. Etiam nec ligula in felis luctus ultrices. Fusce lobortis eros ac nibh porttitor consectetur. Etiam sed ipsum a purus venenatis mollis id et ex. In a sollicitudin nunc, semper auctor neque. Nulla facilisi. Lorem ipsum dolor sit amet, consectetur adipiscing elit. In pharetra odio vitae nisl sollicitudin tempor. Pellentesque eget est mollis, ornare nibh elementum, tincidunt lorem. Ut vestibulum lacus sit amet nulla imperdiet consequat. Curabitur sit amet dui ante.

Duis eros elit, ultricies eu commodo nec, fringilla ut ante. Praesent at quam ac sapien vulputate feugiat non at urna. Quisque semper laoreet augue mollis pulvinar. Phasellus tempor dictum ligula, in consectetur justo blandit a. Donec consequat scelerisque consectetur. Cras eu neque in nisi volutpat scelerisque ac sit amet tortor. Aliquam congue lacinia consectetur. Duis tincidunt pulvinar urna, id tincidunt est suscipit non. Sed faucibus lectus interdum, tristique ex a, commodo nisi. Nunc iaculis malesuada cursus. Proin congue massa nec justo faucibus, et porta massa ultricies. Vivamus sollicitudin turpis in odio tincidunt posuere. Proin augue lectus, faucibus ac massa euismod, pretium elementum diam.

In porta, orci eget convallis congue, nisi velit auctor mauris, eget bibendum ex est quis justo. Vestibulum consequat malesuada dui eu rutrum. Fusce feugiat lorem at nulla ultrices eleifend. Vivamus tellus sapien, dapibus et aliquet vel, bibendum vel dui. Proin volutpat, enim vitae ullamcorper pharetra, nibh eros varius orci, eget bibendum libero nunc in lacus. Fusce non luctus purus. Etiam eu bibendum libero. Vivamus congue, purus ut sollicitudin consequat, ipsum augue rutrum lacus, id ultricies augue justo eu nunc. Fusce porttitor orci venenatis, finibus massa at, ultrices mauris. Sed nec tincidunt mauris. Morbi purus justo, imperdiet nec turpis et, eleifend dictum mi. In vel turpis vitae metus lobortis dapibus ac sit amet urna. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Vestibulum ut lectus aliquet orci tincidunt vehicula.

Proin euismod dictum dui, sit amet auctor augue pulvinar vel. In maximus nisl at quam fringilla, in eleifend arcu hendrerit. Cras hendrerit nibh sed egestas luctus. Morbi rhoncus neque ac enim aliquam convallis. Nam ut pharetra urna. Duis sagittis massa at est sodales sodales. Praesent non nisl nunc. Aenean non elementum arcu, quis hendrerit nunc. Interdum et malesuada fames ac ante ipsum primis in faucibus. Integer eleifend orci ac velit ultrices vestibulum. Donec imperdiet ultricies ligula, vel consequat neque efficitur in. Donec molestie tincidunt lorem, at posuere leo tincidunt vitae.

Maecenas vulputate arcu et congue varius. Quisque et dui placerat, vestibulum arcu a, consectetur orci. Quisque porttitor dignissim tellus, sed aliquam nibh auctor eget. Aliquam erat volutpat. Nam eu bibendum justo. Sed vitae feugiat nulla. Curabitur tempus, nulla quis congue dictum, metus erat auctor metus, sit amet tincidunt nisi diam vitae nisi. Vivamus dictum tincidunt odio ut tincidunt. Nam faucibus aliquet convallis. Ut ac tristique felis. Vestibulum a mi lacus. Etiam auctor felis ut risus vulputate, vel ultrices ante tristique. Praesent congue purus pretium ultricies fermentum. Quisque sit amet est volutpat magna malesuada lobortis eget non justo. In justo turpis, porttitor quis dapibus nec, tincidunt in risus. Pellentesque sagittis tortor risus, ac eleifend ligula fermentum eu.

Suspendisse sed leo id quam lacinia gravida vitae vitae justo. Maecenas malesuada, augue in euismod fermentum, odio nunc mattis neque, nec auctor risus orci sed purus. Integer tincidunt leo felis, et ultricies nunc iaculis at. Sed dictum feugiat massa, sit amet semper ante pulvinar ac. Donec id ultricies purus, non euismod orci. Aliquam erat volutpat. Pellentesque cursus arcu vitae est ultricies mattis. Vivamus at metus magna. Suspendisse a massa eu lorem ullamcorper hendrerit a vel sem. Maecenas bibendum tellus laoreet aliquam consequat. Nulla auctor augue quis malesuada porttitor. Aliquam vehicula hendrerit orci sed tempus. Donec et faucibus odio, nec suscipit lorem. Donec ac leo nisl. Donec at vulputate tortor, vel volutpat velit.

Vestibulum dolor enim, gravida eget leo semper, gravida convallis massa. Nullam vel consectetur nulla. Morbi imperdiet elit id ipsum rutrum, ut commodo odio dapibus. In accumsan augue lorem, eget sollicitudin eros gravida quis. Aenean sed nisi ac purus pretium aliquet a in eros. Vestibulum nec sem in nisl commodo varius in et dolor. Maecenas nec nisi viverra, cursus risus sed, tincidunt arcu. Morbi vel rutrum lectus, non ultrices magna.

Vestibulum sit amet urna vel tortor accumsan tincidunt. In ultricies elit tellus, ac posuere lacus cursus vitae. Donec malesuada leo vitae nibh commodo finibus. Aenean sed ornare nisl, non laoreet turpis. Nunc et tristique nunc. Curabitur mollis enim sed sapien euismod viverra. Integer nec dapibus enim, vitae elementum justo. Integer vitae elit sodales, ullamcorper velit non, mattis sapien. Curabitur et ante eget risus laoreet imperdiet. Morbi venenatis fermentum tellus sed volutpat. Pellentesque mauris nibh, auctor vitae ex nec, suscipit auctor metus. Praesent at purus justo.

Suspendisse metus sem, ornare at libero nec, consectetur accumsan arcu. Duis mauris arcu, iaculis id nibh id, scelerisque sollicitudin dui. Maecenas vel facilisis enim. Phasellus eleifend sed purus at pretium. In rutrum dui eget lobortis euismod. Donec id risus eget nisi finibus scelerisque sed non massa. Integer ultrices nisi nunc, ut semper lectus pretium tincidunt. Nam ipsum ex, fermentum ac fermentum eu, bibendum a elit. Etiam iaculis, arcu non tristique dictum, dui turpis convallis lacus, ut consequat velit massa et nulla. Nullam non fermentum justo. Ut cursus leo eros, a imperdiet metus lobortis sit amet. Nam finibus justo eget purus lobortis, in accumsan magna hendrerit. Maecenas ac est ut nunc blandit tempor. Vivamus nec tincidunt tortor, eget porttitor turpis. Cras vitae mi nec diam tempus consequat.

Sed at dignissim lorem. Sed varius sapien sed ligula maximus, lacinia auctor est aliquam. Proin faucibus massa libero, eu ornare leo aliquet nec. Phasellus eu orci urna. Ut sodales pharetra egestas. Etiam rutrum nisi sodales velit posuere, eget fermentum velit interdum. Vestibulum pellentesque in magna a laoreet.

Ut nec urna ac libero ullamcorper mollis at viverra metus. Curabitur sed dolor vitae mauris fermentum viverra. Suspendisse neque quam, varius nec imperdiet rutrum, hendrerit ac arcu. Maecenas in tellus eu turpis rhoncus congue imperdiet ut neque. In egestas tellus ut diam posuere cursus. Quisque auctor nisi ut augue tempus interdum. Sed dictum viverra mauris vitae ultricies. Vivamus dignissim aliquam neque vitae blandit. Quisque et viverra ligula, ut volutpat est.

Vivamus tempor, ligula sed porta lobortis, nulla dui gravida mauris, nec bibendum ligula magna et turpis. Donec nec sollicitudin enim, ac cursus arcu. Maecenas sit amet maximus orci. Donec ipsum mi, ullamcorper id aliquam luctus, congue ac tellus. Sed ut venenatis nunc. Phasellus aliquam tellus et diam tempus, ut interdum urna tempor. Mauris commodo bibendum augue, at bibendum nunc faucibus a. Phasellus vel nulla finibus, tristique lorem nec, dignissim ex. Aliquam erat volutpat. Donec interdum id ante ultrices tempus. Nulla imperdiet, metus a aliquet molestie, mauris erat eleifend sapien, eget scelerisque nisi ipsum a orci. Quisque porttitor ex vel accumsan pharetra. Donec ornare tincidunt felis ac scelerisque.

Duis et viverra elit, ac ultricies nunc. Curabitur quam nulla, blandit ut magna suscipit, vestibulum ullamcorper sem. Quisque sodales ut urna et varius. Sed condimentum faucibus nisl, sit amet congue tellus vehicula eu. Curabitur lacus dui, tempus eget consectetur a, ultricies vitae dolor. Proin quis odio ac ipsum dapibus ultrices eu nec lorem. Donec id dictum mi. Donec nunc turpis, maximus quis dictum a, eleifend at eros. Sed in imperdiet sapien. Quisque eu porta est, sed faucibus dui. Maecenas dignissim ante finibus velit tempus vestibulum.

Interdum et malesuada fames ac ante ipsum primis in faucibus. Fusce venenatis ac lorem eget iaculis. Cras vehicula odio ac augue euismod, ac efficitur tortor eleifend. In porta mauris sit amet condimentum pellentesque. Aliquam enim sem, pellentesque et purus et, feugiat venenatis risus. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Donec sagittis tempus ullamcorper. Proin commodo justo quis sodales volutpat. Morbi pellentesque erat ac enim tincidunt, sed viverra risus rhoncus. Donec ultricies nulla at lorem auctor, et dictum orci eleifend. Curabitur a augue eget ligula fringilla sodales id et ipsum. Integer mauris nisi, imperdiet mattis pharetra sed, suscipit vitae mauris. Quisque vel odio eu diam maximus congue. Praesent sodales tincidunt lorem, ac egestas libero eleifend eget. Maecenas at odio quis libero convallis suscipit. Mauris et molestie nisl.

Cras scelerisque ornare mi, non ornare ante suscipit et. Donec eros felis, ultricies quis quam non, tristique convallis massa. Etiam ut risus facilisis, rutrum magna vel, mollis ex. Cras ac tellus ipsum. Phasellus faucibus mattis aliquam. Mauris porta ullamcorper nulla quis convallis. Maecenas et ligula ut nisl laoreet rhoncus. Sed bibendum, magna sed semper scelerisque, ante arcu eleifend nibh, non semper libero purus id enim. In ornare risus et nulla lacinia tincidunt. Donec varius felis nec dolor gravida rutrum. Ut nec eleifend lacus. Duis commodo mattis nibh, eu fringilla felis tincidunt venenatis. Mauris sem erat, rhoncus et consequat ac, imperdiet non ante. Integer consectetur convallis risus, quis maximus erat aliquet at. Etiam a massa ut nulla iaculis sagittis. Mauris vestibulum libero a enim imperdiet mattis.

Duis sagittis porttitor magna sit amet porttitor. Fusce pretium venenatis condimentum. Ut aliquam tincidunt mauris nec scelerisque. Praesent tristique odio vitae blandit consectetur. Suspendisse molestie efficitur porta. Vivamus euismod lorem at tempus placerat. Pellentesque placerat nunc ut justo eleifend, sed maximus leo porttitor.

In nec tellus tristique, mattis diam eu, consectetur lorem. Praesent blandit convallis odio, vitae porta risus dignissim a. Vestibulum auctor lacus sit amet elit lacinia malesuada. Donec sed arcu a eros finibus mattis. Ut orci nibh, facilisis non auctor at, vestibulum vitae felis. Nam justo nulla, vulputate ut porttitor sed, pulvinar id erat. Fusce enim ipsum, interdum eget feugiat vel, rutrum pretium tellus. Etiam massa ligula, vestibulum vel lectus at, luctus elementum neque. Sed neque sem, venenatis eu nisl non, lobortis suscipit enim. Sed libero nunc, iaculis condimentum porta in, pharetra id magna. Suspendisse id nisl imperdiet, sagittis sapien id, ullamcorper sapien. Etiam rhoncus massa dui, eget vestibulum purus vulputate tincidunt. Morbi pretium, sem sed varius tincidunt, est arcu volutpat lacus, ut blandit sapien ante at lectus.

Praesent sit amet nulla fringilla, venenatis elit sit amet, ornare lacus. Phasellus condimentum lorem elit, a accumsan mi mattis vitae. Phasellus eros metus, accumsan a vestibulum in, efficitur quis mi. Fusce vitae enim iaculis, interdum tellus in, venenatis ante. Vivamus pulvinar, odio vitae porttitor fermentum, neque nisi lacinia lorem, nec tempus enim massa id nibh. Praesent lobortis sem augue, in luctus ante sagittis a. In vel ultricies sapien. Fusce feugiat a nisi et tempor. Suspendisse potenti. Vivamus non nisi odio. Nunc pharetra dolor varius gravida dictum. Suspendisse iaculis justo nunc, vitae semper odio tempus nec. Nulla facilisi. Vestibulum enim ipsum, condimentum egestas nunc non, bibendum fringilla nisi.

Fusce fermentum velit et eleifend dictum. Suspendisse ornare cursus dolor, vel maximus risus placerat ac. Nunc sodales tortor turpis. Mauris nec fringilla lacus, ut varius nulla. Pellentesque condimentum lorem justo, quis convallis urna placerat sed. Vivamus ac nisl egestas, pulvinar ipsum sed, ultricies lectus. Proin quis sapien ac lacus volutpat ultricies quis ac nisl. Nullam laoreet urna at augue interdum rutrum. Vestibulum eget eros dolor. Sed et consequat lectus. Morbi quis mauris dictum, scelerisque leo vitae, aliquet sem. Ut imperdiet consequat ligula in ultrices. Ut in egestas neque. Donec ullamcorper lorem lacus, vitae mattis purus consectetur ut.

In leo mauris, dignissim eu cursus lacinia, eleifend ac dui. Maecenas congue eget nunc ut consequat. Nam porta blandit sapien non scelerisque. In ut tincidunt urna, sit amet egestas mi. Duis in lorem vitae odio auctor tempus quis eget erat. Nulla placerat ullamcorper congue. Etiam porta erat sit amet magna volutpat porta. Phasellus viverra, turpis at fringilla bibendum, lacus elit tempor risus, non sollicitudin elit mauris ut lectus. Donec imperdiet enim neque, in dignissim nisi auctor id. Duis viverra mauris ac ipsum blandit, vitae auctor ligula bibendum. Vivamus maximus erat id pulvinar rhoncus. Donec dignissim, nisl vitae tincidunt posuere, est urna ultrices ex, blandit aliquam elit odio ac nisi. Pellentesque vitae finibus nisl. Quisque facilisis eu augue suscipit elementum.

Phasellus non fringilla ex. Integer placerat nisl et enim sagittis efficitur. Proin consequat ipsum ac magna fermentum semper. Cras a aliquam nunc. Praesent viverra quam vel ante volutpat tempus. Cras sollicitudin rhoncus eleifend. In hac habitasse platea dictumst. Duis placerat sodales ligula, iaculis semper justo ultrices at. Praesent rutrum porttitor sodales. Fusce condimentum ac risus vel facilisis. Praesent non dui leo. Aenean sit amet metus at felis gravida pretium.

Quisque semper at mi eget interdum. In ac sem non augue accumsan maximus vel et odio. Sed ultrices mi vel nunc hendrerit maximus. Phasellus porttitor erat eu sem porttitor, sit amet tempus erat pellentesque. Praesent aliquet at mi malesuada condimentum. Praesent nisl sapien, finibus id cursus ut, vehicula a risus. Aenean sollicitudin convallis metus, sed pretium massa iaculis ultrices. Proin efficitur auctor justo nec aliquet. In hac habitasse platea dictumst. Etiam ornare arcu eu odio tempor venenatis. Suspendisse varius nec sem eget sodales. Nam porta mauris sed ipsum convallis bibendum. Pellentesque eleifend maximus dictum. Aliquam ut porttitor augue, quis lobortis purus.

Duis vitae elit elementum turpis volutpat malesuada. Sed in arcu at ante egestas bibendum eget eu nulla. Duis sagittis et neque a sagittis. Vivamus vehicula commodo nulla, a consequat felis volutpat sed. Vivamus aliquam, massa ac sollicitudin molestie, tortor dui tempor nisi, non venenatis arcu lectus at leo. Phasellus tristique fermentum mattis. Mauris ac lacus nec erat eleifend faucibus. Proin sodales erat lacus, vitae porttitor lorem suscipit at. Donec ac sapien ultricies, vulputate sem ut, pulvinar sapien.

Cras in dui faucibus, fermentum eros ac, semper nisi. Nunc vitae nunc metus. Nulla sed velit et nisi congue maximus non at lacus. Phasellus vel massa ante. Mauris varius dui vitae risus vehicula feugiat. Donec ultrices mollis erat, laoreet imperdiet elit dignissim vitae. Aenean in pulvinar ipsum. Cras dignissim laoreet lacus eu euismod. Ut porta viverra vestibulum. Fusce ultrices augue odio.

Curabitur mi dui, fringilla ac sem eu, varius tempus turpis. Mauris ex augue, viverra vitae dolor ut, hendrerit faucibus ante. Nam nulla nisl, congue ac erat fringilla, rhoncus aliquet nulla. Maecenas nibh ipsum, bibendum eu erat eu, pharetra vulputate magna. Nullam sed venenatis erat. Ut vitae ex erat. Etiam tincidunt ante vel enim dapibus lacinia. Pellentesque feugiat accumsan nunc. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Vivamus a mauris vitae sem volutpat viverra fermentum quis tellus.

Vivamus neque purus, dignissim mattis ligula id, pretium placerat mi. Vivamus tincidunt risus quis sodales tempor. Nullam luctus venenatis risus, commodo molestie diam vehicula sed. Etiam sagittis sapien ipsum, et feugiat ante posuere vitae. Praesent vitae dolor sit amet eros posuere maximus ac ac ipsum. Curabitur lectus enim, cursus fermentum accumsan nec, egestas id felis. Vestibulum vitae viverra urna. Aenean viverra posuere ante quis sagittis.

Nunc placerat congue congue. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam imperdiet tristique commodo. Pellentesque accumsan suscipit ullamcorper. Donec vitae ligula ante. Quisque sed imperdiet felis. Sed porta eget libero vel blandit. Vivamus tempor a ex et vulputate. Nam vel neque leo. In pretium nisl at sem placerat tristique. Morbi sed metus maximus, laoreet ante eu, aliquet tortor.

Nulla quis aliquet lectus, a consequat leo. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Ut mattis auctor orci, vitae blandit est mattis bibendum. Sed placerat orci a augue euismod gravida. Cras sed congue ligula. Vivamus efficitur lacus neque, vel elementum erat faucibus ut. Fusce tincidunt consequat massa, a vestibulum sapien suscipit at. Donec interdum iaculis porttitor. Nullam sit amet nunc pharetra, consectetur diam sit amet, blandit enim. Quisque sit amet nunc dolor. Ut eros purus, euismod id pharetra sed, scelerisque nec metus.

Integer varius quis risus at sagittis. Ut sed nisl non orci dapibus tempor. Ut egestas eu est a fermentum. Etiam rhoncus aliquet magna. Aenean eleifend purus a mattis vestibulum. Sed sed lacinia eros, luctus auctor dolor. Nulla arcu nisi, finibus non sem sed, feugiat scelerisque massa. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Fusce sagittis iaculis convallis. Pellentesque lacinia aliquam urna, ac egestas enim commodo at. Duis ligula turpis, ultricies ac mi in, laoreet aliquet tortor. Nullam at lectus massa. Fusce suscipit libero non eros placerat posuere.

Fusce vel lorem rutrum, sollicitudin libero in, lacinia justo. Vivamus metus mi, imperdiet eu nisl et, varius sollicitudin quam. Vestibulum molestie feugiat est, in aliquet mauris. Nulla hendrerit ligula sit amet justo varius, ac vehicula sem congue. Aenean ultricies nunc metus, vel lacinia lectus finibus quis. Etiam volutpat pellentesque ullamcorper. Aliquam convallis tempor libero molestie commodo. Phasellus ut magna eget magna fringilla fringilla et vel odio. Aenean bibendum sem at eros condimentum, vitae egestas nisi semper. Vivamus eu urna mauris. Ut vel congue ex. Aliquam hendrerit est ac risus dignissim fermentum. Sed sed imperdiet arcu. Sed sem massa, egestas eu pellentesque nec, lacinia eget orci. Nam a pellentesque leo. Suspendisse potenti.

Morbi vestibulum aliquam ultricies. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec diam est, feugiat nec sapien in, fringilla eleifend ante. Nulla laoreet efficitur enim, ultricies tristique arcu ultricies ut. Etiam diam urna, placerat id dui et, hendrerit fringilla elit. Maecenas et pharetra elit. Curabitur mollis lorem placerat neque rhoncus congue. Maecenas porta nec nibh eget auctor. Nulla rhoncus tristique iaculis. Vivamus congue lacinia elementum. Aliquam eleifend mauris eu turpis efficitur tincidunt.

Quisque diam arcu, lobortis ut egestas at, ultricies venenatis quam. Suspendisse eros orci, interdum vitae porttitor sit amet, pulvinar eu lectus. Nullam sollicitudin finibus mollis. Vivamus nec lectus a felis posuere bibendum quis elementum enim. Duis eu fermentum lorem. Quisque sed mi ex. Ut sagittis ultrices sapien sit amet porta. Sed id ante nibh. Ut pulvinar magna eu ornare hendrerit. Curabitur id metus at justo eleifend efficitur vel eu nunc.

Morbi ullamcorper dui et rutrum molestie. Sed vitae urna in felis aliquet congue luctus vel metus. Suspendisse leo velit, venenatis sed aliquam eget, euismod non ante. Phasellus vulputate neque nisl, ac vestibulum justo consectetur quis. Sed nec tristique ligula. Nulla convallis at arcu eu dapibus. Vestibulum et risus dapibus, pellentesque lacus ac, porttitor urna.

Integer at lorem semper nibh aliquet iaculis. Maecenas in lectus ultrices, interdum dui ac, rutrum dolor. Etiam sit amet purus id purus porttitor volutpat. In hac habitasse platea dictumst. Nunc mi purus, viverra nec velit nec, molestie volutpat sapien. Maecenas egestas in dui sed dapibus. Praesent nec maximus nibh, nec luctus quam. Vivamus accumsan felis eros, eu fringilla justo ullamcorper nec. Sed eu consectetur leo, et iaculis odio. Curabitur iaculis, eros et ornare tempus, purus tortor ultrices dolor, nec varius elit sem et turpis.

Maecenas vel fermentum quam, id blandit lectus. Aenean fermentum, ligula sit amet cursus lacinia, felis metus ullamcorper metus, et porttitor tortor metus et velit. Donec facilisis sem mollis, vestibulum libero ut, tempus lorem. Duis elit est, semper a tortor sit amet, pulvinar tempor purus. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Vivamus hendrerit mattis iaculis. In lobortis, magna quis pretium consectetur, velit diam cursus arcu, vel viverra quam quam ut velit. Suspendisse eleifend metus id ullamcorper pretium. Nam laoreet, erat a facilisis finibus, tortor sem faucibus ipsum, ac ornare turpis nunc vel diam. Mauris eget ultricies nulla, placerat congue velit. Proin ligula massa, maximus nec libero ac, sodales finibus nunc. Maecenas a magna pellentesque, convallis enim at, tempor est. In fringilla commodo ipsum. Etiam consequat ultricies dolor quis molestie. Morbi luctus libero est, ac suscipit ante volutpat scelerisque.";
    c.bench_function("parse http req", |b| b.iter(|| build_and_parse_req(http.to_vec())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
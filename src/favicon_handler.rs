use std::fs::File;
use std::path::{PathBuf, Path};
use std::io::Read;

use hyper::uri::RequestUri::AbsolutePath;
use hyper::method::Method::{Get, Head, Options};
use hyper::status::StatusCode;
use hyper::header;
use hyper::net;

use request::Request;
use response::Response;
use middleware::{Continue, Middleware, MiddlewareResult};
use mimes::MediaType;

pub struct FaviconHandler {
    icon: Vec<u8>,
    icon_path: PathBuf, // Is it useful to log where in-memory favicon came from every request?
}

impl Middleware for FaviconHandler {
    fn invoke<'a, 'server>(&'a self, req: &mut Request<'a, 'server>, res: Response<'a, net::Fresh>)
            -> MiddlewareResult<'a> {
        if FaviconHandler::is_favicon_request(req) {
            self.handle_request(req, res)
        } else {
            Ok(Continue(res))
        }
    }
}

impl FaviconHandler {
    /// Create a new middleware to serve an /favicon.ico file from an in-memory cache.
    /// The file is only read from disk once when the server starts.
    ///
    /// # Examples
    /// ```{rust,no_run}
    /// use nickel::{Nickel, FaviconHandler};
    /// let mut server = Nickel::new();
    ///
    /// server.utilize(FaviconHandler::new("/path/to/ico/file"));
    /// ```
    pub fn new<P: AsRef<Path>>(icon_path: P) -> FaviconHandler {
        let icon_path = icon_path.as_ref().to_path_buf();
        let mut icon = vec![];
        File::open(&icon_path).unwrap().read_to_end(&mut icon).unwrap();

        FaviconHandler {
            // Fail when favicon cannot be read. Better error message though?
            icon: icon,
            icon_path: icon_path,
        }
    }

    #[inline]
    pub fn is_favicon_request(req: &Request) -> bool {
        match req.origin.uri {
            AbsolutePath(ref path) => &**path == "/favicon.ico",
            _                      => false
        }
    }

    pub fn handle_request<'a>(&self, req: &Request, mut res: Response<'a>) -> MiddlewareResult<'a> {
        match req.origin.method {
            Get | Head => {
                self.send_favicon(req, res)
            },
            Options => {
                res.set(StatusCode::Ok);
                res.set(header::Allow(vec![Get, Head, Options]));
                res.send("")
            },
            _ => {
                res.set(StatusCode::MethodNotAllowed);
                res.set(header::Allow(vec![Get, Head, Options]));
                res.send("")
            }
        }
    }

    pub fn send_favicon<'a, 'server>(&self, req: &Request, mut res: Response<'a>) -> MiddlewareResult<'a> {
        debug!("{:?} {:?}", req.origin.method, self.icon_path.display());
        res.set(MediaType::Ico);
        res.send(&*self.icon)
    }
}

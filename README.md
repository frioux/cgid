# cgid

`cgid` is a [UCSPI](http://cr.yp.to/proto/ucspi.txt) compatible
[CGI](https://www.ietf.org/rfc/rfc3875) server.  It currently supports exactly
one script, though there are plans to support a directory.

## Usage

Here are a couple examples of using `cgid` with `UCSPI`

### [`nosh`](http://homepage.ntlworld.com/jonathan.deboynepollard/Softwares/nosh.html)

```
#!/bin/nosh
tcp-socket-listen 127.0.0.1 6000
tcp-socket-accept --no-delay
cgid
www/cgi-bin/my-cgi-script
```

### [`s6`](http://skarnet.org/software/s6/index.html)

```
#!/bin/execlineb
s6-tcpserver 127.0.0.1 6000
cgid
www/cgi-bin/my-cgi-script
```

(other examples are very welcome)

## Description

`cgid` implements a (hopefully reasonable) subset of the `CGI` protocol.  It
almost surely has glaring missing features, as I have only run a single program
underneath it.  Bug reports and patches are warmly welcome.

## Known Bugs and Limitations

### First response header **MUST** be `Status`

As far as I can tell the CGI specification allows the response headers to be in
any order.  Because all of my applications set the Status first I have not
written the code to buffer the other headers before the Status is set.  As it
stands any headers set before status will be discarded.  I expect to resolve
this when I get a chance.

### `SCRIPT_NAME` is hardcoded to an empty string

If my reading of [`RFC3875`](https://www.ietf.org/rfc/rfc3875) is correctly,
`SCRIPT_NAME` should be which script is being run.  Given that this server runs
exactly one script, I've decided that setting this environment variable is
unimportant.  If I ever support a directory instead of a single script I'll
resolve this.

### Only `HTTP/1.*` is supported

Technically `HTTP/1.0` supports `HTTP/0.9`, which supports responses with no
response code or headers.  This is not supported for simplicity.

### Response `Content-Length` is unsupported

For both performance and simplicity, the response body is streamed from the
application to the client, so the `Content-Length` is unknown and cannot be sent
to the client unless the application set the response header itself.

### Esoteric Error

There is one possible error that is impossible to report to the client
correctly.  Specifically, if there is an error while streaming the response body
to the client from the application, returning a `500 Internal Server Error` is
likely impossible as the applications status has already been set.

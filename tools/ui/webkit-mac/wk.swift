import Cocoa
import WebKit

// usage: wk <url> <body.js> <outdir> <save-file> <width> <height>
let a = CommandLine.arguments
let url = URL(string: a[1])!
let body = try! String(contentsOfFile: a[2], encoding: .utf8)
let outDir = a[3]
let save = try! String(contentsOfFile: a[4], encoding: .utf8)
let W = Double(a[5])!, H = Double(a[6])!
try? FileManager.default.createDirectory(atPath: outDir, withIntermediateDirectories: true)

final class Delegate: NSObject, WKNavigationDelegate {
  var web: WKWebView!
  func webView(_ w: WKWebView, didFinish nav: WKNavigation!) {
    print("loaded", w.url?.absoluteString ?? "")
    w.callAsyncJavaScript(body, arguments: [:], in: nil, in: .page) { r in
      switch r {
      case .success(let v):
        if let d = v as? [String: Any] {
          if let logs = d["log"] as? [String] { for l in logs { print(l) } }
          if let imgs = d["images"] as? [String: String] {
            for (n, u) in imgs {
              if let comma = u.firstIndex(of: ","), let data = Data(base64Encoded: String(u[u.index(after: comma)...])) {
                try? data.write(to: URL(fileURLWithPath: "\(outDir)/\(n).png"))
              }
            }
          }
        }
        print("done")
      case .failure(let e): print("error", e)
      }
      exit(0)
    }
  }
  func webView(_ w: WKWebView, didFail nav: WKNavigation!, withError e: Error) { print("nav fail", e); exit(1) }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let cfg = WKWebViewConfiguration()
let ucc = WKUserContentController()
let esc = save.replacingOccurrences(of: "\\", with: "\\\\").replacingOccurrences(of: "`", with: "\\`").replacingOccurrences(of: "$", with: "\\$")
ucc.addUserScript(WKUserScript(source: "try{localStorage.setItem('legatum.autosave.v1', `\(esc)`);localStorage.setItem('legatum.first-game-hints.v1', JSON.stringify({enabled:false,seen:[]}));}catch(e){}", injectionTime: .atDocumentStart, forMainFrameOnly: true))
cfg.userContentController = ucc
let win = NSWindow(contentRect: NSRect(x: 0, y: 0, width: W, height: H), styleMask: [.borderless], backing: .buffered, defer: false)
let web = WKWebView(frame: NSRect(x: 0, y: 0, width: W, height: H), configuration: cfg)
let del = Delegate(); del.web = web
web.navigationDelegate = del
win.contentView = web
win.alphaValue = 0.02
win.ignoresMouseEvents = true
win.orderFrontRegardless()
web.load(URLRequest(url: url))
app.run()

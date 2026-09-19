// Debug QA only. WKWebView renders its own document into a fresh bitmap; no
// screen/window-server capture, desktop coordinates, focus or image upscaling.
#import <AppKit/AppKit.h>
#import <WebKit/WebKit.h>
#include <math.h>
#include <stdint.h>

bool kindling_qa_supported(void) {
    if (@available(macOS 14.0, *)) return true;
    return false;
}

typedef void (*QAReply)(uint64_t request, const char *json);

static void replyJSON(uint64_t request, QAReply reply, NSDictionary *value) {
    NSData *data = [NSJSONSerialization dataWithJSONObject:value options:0 error:nil];
    NSString *json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
    reply(request, json.UTF8String);
}

static void fail(uint64_t request, QAReply reply, NSString *message) {
    replyJSON(request, reply, @{ @"error": message ?: @"WebKit snapshot failed" });
}

// Called on the main thread by Tauri's with_webview. The callback copies the
// JSON before this autorelease pool/block returns; it never retains ObjC data.
void kindling_qa_request(void *handle, uint64_t request, const char *operation,
                         uint32_t width, uint32_t height, uint32_t scale, QAReply reply) {
    @autoreleasepool {
        WKWebView *view = (__bridge WKWebView *)handle;
        if (![NSThread isMainThread] || !view.window) {
            fail(request, reply, @"Snapshot requires an attached WKWebView on the main thread");
            return;
        }
        if (strcmp(operation, "resize") == 0) {
            // The QA window is borderless, ordered out and ignores mouse input.
            // Its logical canvas is independent of any connected screen's bounds.
            [view.window setContentSize:NSMakeSize(width, height)];
            [view setFrameSize:NSMakeSize(width, height)];
            [view layoutSubtreeIfNeeded];
            replyJSON(request, reply, @{ @"width": @(view.bounds.size.width),
                                        @"height": @(view.bounds.size.height) });
            return;
        }
        if (fabs(view.bounds.size.width - width) > 0.01 ||
            fabs(view.bounds.size.height - height) > 0.01) {
            fail(request, reply, @"Webview viewport changed before snapshot");
            return;
        }

        // WebKit multiplies snapshotWidth by its device scale when rasterizing.
        // Compensate BEFORE rendering, so both 1x and 2x displays produce exactly
        // width*scale pixels. Never resize the resulting bitmap to pass a check.
        [view evaluateJavaScript:@"window.devicePixelRatio" completionHandler:^(id value, NSError *error) {
            if (error || ![value isKindOfClass:[NSNumber class]]) {
                fail(request, reply, @"Could not read WebKit device scale");
                return;
            }
            double deviceScale = [value doubleValue];
            if (!isfinite(deviceScale) || deviceScale <= 0 || deviceScale > 8) {
                fail(request, reply, @"Invalid WebKit device scale");
                return;
            }
            WKSnapshotConfiguration *configuration = [WKSnapshotConfiguration new];
            configuration.rect = CGRectMake(0, 0, width, height);
            configuration.snapshotWidth = @((double)width * scale / deviceScale);
            configuration.afterScreenUpdates = NO;
            [view takeSnapshotWithConfiguration:configuration completionHandler:^(NSImage *image, NSError *snapshotError) {
                if (snapshotError || !image) {
                    fail(request, reply, snapshotError.localizedDescription);
                    return;
                }
                CGImageRef cgImage = [image CGImageForProposedRect:NULL context:nil hints:nil];
                if (!cgImage || CGImageGetWidth(cgImage) != width * scale ||
                    CGImageGetHeight(cgImage) != height * scale) {
                    fail(request, reply, @"WebKit returned unexpected pixel dimensions; no resize attempted");
                    return;
                }
                if (fabs(view.bounds.size.width - width) > 0.01 ||
                    fabs(view.bounds.size.height - height) > 0.01) {
                    fail(request, reply, @"Webview viewport changed during snapshot");
                    return;
                }
                NSBitmapImageRep *bitmap = [[NSBitmapImageRep alloc] initWithCGImage:cgImage];
                NSData *png = [bitmap representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
                if (!png.length) {
                    fail(request, reply, @"PNG encoding failed");
                    return;
                }
                replyJSON(request, reply, @{
                    @"profile": @"wkwebview-snapshot-2x-png-v1",
                    @"engine": @"WKWebView.takeSnapshot",
                    @"pixels": @[@(CGImageGetWidth(cgImage)), @(CGImageGetHeight(cgImage))],
                    @"viewport": @{ @"width": @(width), @"height": @(height), @"dpr": @(deviceScale) },
                    @"scale": @(scale), @"windowVisible": @(view.window.isVisible),
                    @"windowFocused": @(view.window.isKeyWindow),
                    @"pngBase64": [png base64EncodedStringWithOptions:0]
                });
            }];
        }];
    }
}

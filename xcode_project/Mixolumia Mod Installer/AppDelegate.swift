//
//  AppDelegate.swift
//  Mixolumia Mod Installer
//
//  Created by m s on 9/1/20.
//  Copyright © 2020 maddymakesgames. All rights reserved.
//

import Cocoa

@NSApplicationMain
class AppDelegate: NSObject, NSApplicationDelegate {

    var file: Bool = false;

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        // Insert code here to initialize your application
        if(!file && !is_installed()) {
            gen_install_data();
            exit(1);
        } else {
            exit(1);
        }
    }

    func applicationWillTerminate(_ aNotification: Notification) {
        // Insert code here to tear down your application
    }

    func application(_ sender: NSApplication, openFile filename: String) -> Bool {
        install_file(filename);
        file = true;
        return true;
    }

}

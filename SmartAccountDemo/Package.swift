// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.
// Swift Package: SmartAccountDemo

import PackageDescription;

let package = Package(
    name: "SmartAccountDemo",
    platforms: [
        .iOS(.v13),
        .macOS(.v10_15)
    ],
    products: [
        .library(
            name: "SmartAccountDemo",
            targets: ["SmartAccountDemo"]
        )
    ],
    dependencies: [ ],
    targets: [
        .binaryTarget(name: "RustFramework", path: "./RustFramework.xcframework"),
        .target(
            name: "SmartAccountDemo",
            dependencies: [
                .target(name: "RustFramework")
            ]
        ),
    ]
)
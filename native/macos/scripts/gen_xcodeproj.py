#!/usr/bin/env python3
"""Deterministically generates JustSlide.xcodeproj (project.pbxproj + shared
scheme) from the files on disk, mirroring project.yml. Run after adding or
removing Swift files:  python3 native/macos/scripts/gen_xcodeproj.py
"""
import hashlib, os
from pathlib import Path

HERE = Path(__file__).resolve().parents[1]
APP_SRC = sorted(p.name for p in (HERE / "JustSlide").glob("*.swift"))
TEST_SRC = sorted(p.name for p in (HERE / "JustSlideTests").glob("*.swift"))


def oid(*parts):
    return hashlib.md5("/".join(parts).encode()).hexdigest()[:24].upper()


P = oid("project")
T_APP, T_TEST = oid("target", "app"), oid("target", "test")
G_MAIN, G_APP, G_TEST, G_PROD, G_CORE = oid("g", "main"), oid("g", "app"), oid("g", "test"), oid("g", "products"), oid("g", "core")
PROD_APP, PROD_TEST = oid("prod", "app"), oid("prod", "test")
PH = {k: oid("phase", k) for k in ["app-src", "app-fw", "app-res", "app-rust", "app-chrome", "test-src", "test-fw", "test-res"]}
CL = {k: oid("cfglist", k) for k in ["project", "app", "test"]}
CFG = {(t, c): oid("cfg", t, c) for t in ["project", "app", "test"] for c in ["Debug", "Release"]}
DEP, PROXY = oid("dep"), oid("proxy")

files = {}  # fileRef id -> (path, type, group)
build = []  # (buildFile id, fileRef id, phase)
for name in APP_SRC:
    fr = oid("file", "app", name); files[fr] = (name, "sourcecode.swift", G_APP); build.append((oid("bf", "app", name), fr, "app-src"))
for name in TEST_SRC:
    fr = oid("file", "test", name); files[fr] = (name, "sourcecode.swift", G_TEST); build.append((oid("bf", "test", name), fr, "test-src"))
for name, typ in [("Info.plist", "text.plist.xml"), ("JustSlide.entitlements", "text.plist.entitlements"), ("JustSlide-Bridging-Header.h", "sourcecode.c.h")]:
    files[oid("file", "app", name)] = (name, typ, G_APP)
HDR = oid("file", "core", "justslide.h"); files[HDR] = ("include/justslide.h", "sourcecode.c.h", G_CORE)

common = {
    "ALWAYS_SEARCH_USER_PATHS": "NO", "CLANG_ENABLE_MODULES": "YES", "MACOSX_DEPLOYMENT_TARGET": "14.0", "SDKROOT": "macosx",
    "SWIFT_VERSION": "5.0", "CODE_SIGN_IDENTITY": '"-"', "CODE_SIGN_STYLE": "Automatic", "ENABLE_USER_SCRIPT_SANDBOXING": "NO",
}
dbg = {"DEBUG_INFORMATION_FORMAT": "dwarf", "ONLY_ACTIVE_ARCH": "YES", "SWIFT_ACTIVE_COMPILATION_CONDITIONS": "DEBUG", "SWIFT_OPTIMIZATION_LEVEL": '"-Onone"', "ENABLE_TESTABILITY": "YES", "GCC_PREPROCESSOR_DEFINITIONS": '("DEBUG=1", "$(inherited)")'}
rel = {"DEBUG_INFORMATION_FORMAT": '"dwarf-with-dsym"', "SWIFT_COMPILATION_MODE": "wholemodule", "SWIFT_OPTIMIZATION_LEVEL": '"-O"'}
app = {
    "PRODUCT_NAME": "JustSlide", "PRODUCT_BUNDLE_IDENTIFIER": "app.justslide.JustSlide.dev", "INFOPLIST_FILE": "JustSlide/Info.plist",
    "CODE_SIGN_ENTITLEMENTS": "JustSlide/JustSlide.entitlements", "ENABLE_HARDENED_RUNTIME": "YES", "COMBINE_HIDPI_IMAGES": "YES",
    "SWIFT_OBJC_BRIDGING_HEADER": '"JustSlide/JustSlide-Bridging-Header.h"', "HEADER_SEARCH_PATHS": '"$(SRCROOT)/JustSlideCore/include"',
    "LIBRARY_SEARCH_PATHS": '"$(BUILT_PRODUCTS_DIR)"', "OTHER_LDFLAGS": '"-ljustslide_ffi"', "LD_RUNPATH_SEARCH_PATHS": '"@executable_path/../Frameworks"',
}
test = {
    "PRODUCT_NAME": "JustSlideTests", "PRODUCT_BUNDLE_IDENTIFIER": "app.justslide.JustSlide.dev.tests", "GENERATE_INFOPLIST_FILE": "YES",
    "TEST_HOST": '"$(BUILT_PRODUCTS_DIR)/JustSlide.app/Contents/MacOS/JustSlide"', "BUNDLE_LOADER": '"$(TEST_HOST)"',
    "SWIFT_OBJC_BRIDGING_HEADER": '"JustSlide/JustSlide-Bridging-Header.h"', "HEADER_SEARCH_PATHS": '"$(SRCROOT)/JustSlideCore/include"',
}


def settings(d):
    return "".join(f"\t\t\t\t{k} = {v};\n" for k, v in sorted(d.items()))


out = ["// !$*UTF8*$!\n{\n\tarchiveVersion = 1;\n\tclasses = {\n\t};\n\tobjectVersion = 56;\n\tobjects = {\n"]
out.append("\n/* Begin PBXBuildFile section */\n")
for bf, fr, ph in build:
    out.append(f"\t\t{bf} /* {files[fr][0]} */ = {{isa = PBXBuildFile; fileRef = {fr}; }};\n")
out.append("/* End PBXBuildFile section */\n\n/* Begin PBXContainerItemProxy section */\n")
out.append(f"\t\t{PROXY} = {{isa = PBXContainerItemProxy; containerPortal = {P}; proxyType = 1; remoteGlobalIDString = {T_APP}; remoteInfo = JustSlide; }};\n")
out.append("/* End PBXContainerItemProxy section */\n\n/* Begin PBXFileReference section */\n")
for fr, (path, typ, _) in files.items():
    out.append(f"\t\t{fr} = {{isa = PBXFileReference; lastKnownFileType = {typ}; path = \"{path}\"; sourceTree = \"<group>\"; }};\n")
out.append(f"\t\t{PROD_APP} = {{isa = PBXFileReference; explicitFileType = wrapper.application; includeInIndex = 0; path = JustSlide.app; sourceTree = BUILT_PRODUCTS_DIR; }};\n")
out.append(f"\t\t{PROD_TEST} = {{isa = PBXFileReference; explicitFileType = wrapper.cfbundle; includeInIndex = 0; path = JustSlideTests.xctest; sourceTree = BUILT_PRODUCTS_DIR; }};\n")
out.append("/* End PBXFileReference section */\n\n/* Begin PBXFrameworksBuildPhase section */\n")
for k in ["app-fw", "test-fw"]:
    out.append(f"\t\t{PH[k]} = {{isa = PBXFrameworksBuildPhase; buildActionMask = 2147483647; files = (); runOnlyForDeploymentPostprocessing = 0; }};\n")
out.append("/* End PBXFrameworksBuildPhase section */\n\n/* Begin PBXGroup section */\n")


def group(gid, name, children, path=None):
    p = f' path = "{path}";' if path else ""
    n = f' name = "{name}";' if name and not path else ""
    return f"\t\t{gid} = {{isa = PBXGroup; children = ({', '.join(children)}, );{n}{p} sourceTree = \"<group>\"; }};\n"


out.append(group(G_MAIN, None, [G_APP, G_TEST, G_CORE, G_PROD]))
out.append(group(G_APP, None, [f for f, v in files.items() if v[2] == G_APP], "JustSlide"))
out.append(group(G_TEST, None, [f for f, v in files.items() if v[2] == G_TEST], "JustSlideTests"))
out.append(group(G_CORE, None, [HDR], "JustSlideCore"))
out.append(group(G_PROD, "Products", [PROD_APP, PROD_TEST]))
out.append("/* End PBXGroup section */\n\n/* Begin PBXNativeTarget section */\n")
out.append(f"\t\t{T_APP} = {{isa = PBXNativeTarget; buildConfigurationList = {CL['app']}; buildPhases = ({PH['app-rust']}, {PH['app-src']}, {PH['app-fw']}, {PH['app-res']}, {PH['app-chrome']}, ); buildRules = (); dependencies = (); name = JustSlide; productName = JustSlide; productReference = {PROD_APP}; productType = \"com.apple.product-type.application\"; }};\n")
out.append(f"\t\t{T_TEST} = {{isa = PBXNativeTarget; buildConfigurationList = {CL['test']}; buildPhases = ({PH['test-src']}, {PH['test-fw']}, {PH['test-res']}, ); buildRules = (); dependencies = ({DEP}, ); name = JustSlideTests; productName = JustSlideTests; productReference = {PROD_TEST}; productType = \"com.apple.product-type.bundle.unit-test\"; }};\n")
out.append("/* End PBXNativeTarget section */\n\n/* Begin PBXProject section */\n")
out.append(f"\t\t{P} = {{isa = PBXProject; attributes = {{BuildIndependentTargetsInParallel = 1; LastSwiftUpdateCheck = 2600; LastUpgradeCheck = 2600; TargetAttributes = {{{T_TEST} = {{TestTargetID = {T_APP}; }}; }}; }}; buildConfigurationList = {CL['project']}; compatibilityVersion = \"Xcode 14.0\"; developmentRegion = en; hasScannedForEncodings = 0; knownRegions = (en, Base, ); mainGroup = {G_MAIN}; productRefGroup = {G_PROD}; projectDirPath = \"\"; projectRoot = \"\"; targets = ({T_APP}, {T_TEST}, ); }};\n")
out.append("/* End PBXProject section */\n\n/* Begin PBXResourcesBuildPhase section */\n")
for k in ["app-res", "test-res"]:
    out.append(f"\t\t{PH[k]} = {{isa = PBXResourcesBuildPhase; buildActionMask = 2147483647; files = (); runOnlyForDeploymentPostprocessing = 0; }};\n")
out.append("/* End PBXResourcesBuildPhase section */\n\n/* Begin PBXShellScriptBuildPhase section */\n")
for k, name, script in [("app-rust", "Build Rust core", '/bin/bash \\"$SRCROOT/scripts/build-rust-core.sh\\"'), ("app-chrome", "Build and copy chrome UI", '/bin/bash \\"$SRCROOT/scripts/copy-chrome.sh\\"')]:
    out.append(f"\t\t{PH[k]} = {{isa = PBXShellScriptBuildPhase; alwaysOutOfDate = 1; buildActionMask = 2147483647; files = (); inputPaths = (); name = \"{name}\"; outputPaths = (); runOnlyForDeploymentPostprocessing = 0; shellPath = /bin/bash; shellScript = \"{script}\\n\"; showEnvVarsInLog = 0; }};\n")
out.append("/* End PBXShellScriptBuildPhase section */\n\n/* Begin PBXSourcesBuildPhase section */\n")
for k in ["app-src", "test-src"]:
    ids = [bf for bf, _, ph in build if ph == k]
    out.append(f"\t\t{PH[k]} = {{isa = PBXSourcesBuildPhase; buildActionMask = 2147483647; files = ({', '.join(ids)}, ); runOnlyForDeploymentPostprocessing = 0; }};\n")
out.append("/* End PBXSourcesBuildPhase section */\n\n/* Begin PBXTargetDependency section */\n")
out.append(f"\t\t{DEP} = {{isa = PBXTargetDependency; target = {T_APP}; targetProxy = {PROXY}; }};\n")
out.append("/* End PBXTargetDependency section */\n\n/* Begin XCBuildConfiguration section */\n")
for t, extra in [("project", common), ("app", app), ("test", test)]:
    for c, mode in [("Debug", dbg), ("Release", rel)]:
        s = dict(extra) if t != "project" else {**common, **mode}
        out.append(f"\t\t{CFG[(t, c)]} = {{\n\t\t\tisa = XCBuildConfiguration;\n\t\t\tbuildSettings = {{\n{settings(s)}\t\t\t}};\n\t\t\tname = {c};\n\t\t}};\n")
out.append("/* End XCBuildConfiguration section */\n\n/* Begin XCConfigurationList section */\n")
for t in ["project", "app", "test"]:
    out.append(f"\t\t{CL[t]} = {{isa = XCConfigurationList; buildConfigurations = ({CFG[(t, 'Debug')]}, {CFG[(t, 'Release')]}, ); defaultConfigurationIsVisible = 0; defaultConfigurationName = Release; }};\n")
out.append("/* End XCConfigurationList section */\n\t};\n\trootObject = " + P + ";\n}\n")

proj = HERE / "JustSlide.xcodeproj"
(proj / "xcshareddata/xcschemes").mkdir(parents=True, exist_ok=True)
(proj / "project.pbxproj").write_text("".join(out))
ref = lambda t, n, p: f'<BuildableReference BuildableIdentifier = "primary" BlueprintIdentifier = "{t}" BuildableName = "{p}" BlueprintName = "{n}" ReferencedContainer = "container:JustSlide.xcodeproj"></BuildableReference>'
(proj / "xcshareddata/xcschemes/JustSlide.xcscheme").write_text(f"""<?xml version="1.0" encoding="UTF-8"?>
<Scheme LastUpgradeVersion = "2600" version = "1.7">
   <BuildAction parallelizeBuildables = "YES" buildImplicitDependencies = "YES">
      <BuildActionEntries>
         <BuildActionEntry buildForTesting = "YES" buildForRunning = "YES" buildForProfiling = "YES" buildForArchiving = "YES" buildForAnalyzing = "YES">
            {ref(T_APP, "JustSlide", "JustSlide.app")}
         </BuildActionEntry>
      </BuildActionEntries>
   </BuildAction>
   <TestAction buildConfiguration = "Debug" selectedDebuggerIdentifier = "Xcode.DebuggerFoundation.Debugger.LLDB" selectedLauncherIdentifier = "Xcode.DebuggerFoundation.Launcher.LLDB" shouldUseLaunchSchemeArgsEnv = "YES">
      <Testables>
         <TestableReference skipped = "NO">
            {ref(T_TEST, "JustSlideTests", "JustSlideTests.xctest")}
         </TestableReference>
      </Testables>
   </TestAction>
   <LaunchAction buildConfiguration = "Debug" selectedDebuggerIdentifier = "Xcode.DebuggerFoundation.Debugger.LLDB" selectedLauncherIdentifier = "Xcode.DebuggerFoundation.Launcher.LLDB" launchStyle = "0" useCustomWorkingDirectory = "NO" ignoresPersistentStateOnLaunch = "NO" debugDocumentVersioning = "YES" debugServiceExtension = "internal" allowLocationSimulation = "YES">
      <BuildableProductRunnable runnableDebuggingMode = "0">
         {ref(T_APP, "JustSlide", "JustSlide.app")}
      </BuildableProductRunnable>
   </LaunchAction>
   <ArchiveAction buildConfiguration = "Release" revealArchiveInOrganizer = "YES"></ArchiveAction>
</Scheme>
""")
print(f"wrote {proj} ({len(APP_SRC)} app sources, {len(TEST_SRC)} test sources)")

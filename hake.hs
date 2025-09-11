{-# LANGUAGE MultiWayIf    #-}
{-# LANGUAGE UnicodeSyntax #-}

import           Hake

main ∷ IO ()
main = hake $ do

  "clean | clean the project" ∫
    cargo ["clean"] ?> removeDirIfExists targetPath

  "update | update dependencies" ∫ cargo ["update"]

  sunsetExecutable ♯
    cargo <| "build" : buildFlagsSunset False

  "fat | build Sunset  with fat LTO" ∫
       cargo <| "build" : buildFlagsSunset True

  "install | install to system" ◉ [ "fat" ] ∰
    cargo <| "install" : buildFlagsSunset True

  "test | build and test" ◉ [sunsetExecutable] ∰ do
    cargo ["test"]
    cargo ["clippy"]
    rawSystem sunsetExecutable ["--version"]
      >>= checkExitCode

  "restart | restart services" ◉ [ sunsetExecutable ] ∰
    systemctl ["restart", appNameSunset]

  "run | run sunset" ◉ [ sunsetExecutable ] ∰ do
    cargo . (("run" : buildFlagsSunset False) ++) . ("--" :) =<< getHakeArgs

 where
  appNameSunset ∷ String
  appNameSunset = "sunset"

  targetPath ∷ FilePath
  targetPath = "target"

  buildPath ∷ FilePath
  buildPath = targetPath </> "release"

  sunsetFeatures ∷ [String]
  sunsetFeatures = [ ]

  fatArgs ∷ [String]
  fatArgs = [ "--profile"
            , "fat-release" ]

  buildFlagsSunset ∷ Bool -> [String]
  buildFlagsSunset fat =
    let defaultFlags = [ "-p", appNameSunset
                       , "--release", "--features"
                       , intercalate "," sunsetFeatures ]
    in if fat then defaultFlags ++ fatArgs
              else defaultFlags

  sunsetExecutable ∷ FilePath
  sunsetExecutable = buildPath </> appNameSunset

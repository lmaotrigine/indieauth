module Model exposing (Model, Msg(..), get, init)

import Browser exposing (UrlRequest(..))
import Browser.Navigation as Nav
import Http
import App
import Route exposing (Route, routeParser)
import Url exposing (Url)
import Url.Parser as UrlParser

type alias Model =
    { navKey : Nav.Key
    , route : Maybe Route
    , token : Maybe String
    , tokenData : Maybe App.TokenData
    , error : Maybe String
    }

get : Model -> String -> Http.Expect Msg -> Cmd Msg
get model url action =
  App.request
    "GET"
    (Maybe.withDefault "" model.token)
    url
    Http.emptyBody
    action

type Msg
  = ChangeUrl Url
  | ClickLink UrlRequest
  | UpdateToken String
  | SubmitToken
  | ValidateToken (Result Http.Error App.TokenData)
  | ClearError

init : Maybe String -> Url -> Nav.Key -> (Model, Cmd msg)
init token url key =
  ( { navKey = key
    , route = UrlParser.parse routeParser url
    , token = token
    , tokenData = Nothing
    , error = Nothing
    }
    , Nav.pushUrl key "/login"
  )

module Main exposing (main)

import Browser exposing (Document, UrlRequest(..))
import Browser.Navigation as Nav
import Html exposing (a, p, text)
import Html.Attributes exposing (href)
import Html.Events exposing (onClick)
import Http
import Layout
import App
import Model exposing (Model, Msg(..), get, init)
import Page.Index
import Page.Login
import Route exposing (Route(..), routeParser)
import Url
import Url.Parser as UrlParser

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  let
    if_okay : Result Http.Error a -> (a -> (Model, Cmd Msg)) -> (Model, Cmd Msg)
    if_okay result doer =
      case result of
        Ok data ->
          doer data
        
        Err why ->
          ( { model | error = Just <| App.errorToString why }, Cmd.none )
  in
  case msg of
    UpdateToken newToken ->
      ( { model | token = Just newToken }, Cmd.none )

    ChangeUrl url ->
      ( { model | route = UrlParser.parse routeParser url }, Cmd.none )
    
    SubmitToken ->
      ( model
      , Cmd.batch
        [ get model App.tokenIntrospectionURL <|
          App.expectJson ValidateToken App.tokenDecoder
        ]
      )
    
    ValidateToken result ->
      if_okay result <|
        \data ->
          ( { model | tokenData = Just data }
          , Nav.pushUrl model.navKey "/"
          )
    
    ClickLink urlRequest ->
      case urlRequest of
        Internal url ->
          ( model, Nav.pushUrl model.navKey <| Url.toString url )
        
        External url ->
          ( model, Nav.load url )
    
    ClearError ->
      ( { model | error = Nothing }, Cmd.none )

view : Model -> Document Msg
view model =
  case model.error of
    Nothing ->
      case Maybe.withDefault Index model.route of
        Index ->
          Page.Index.view model
        
        Login ->
          Page.Login.view model
        
        _ ->
          Layout.template "Oh noes" [ p [] [ text "TODO: implement this 404 page" ] ]
    
    Just why ->
      Layout.basic
        "Error"
        [ p [][ text why, text ". Please clear the error to proceed." ]
        , a [ onClick ClearError, href "/" ] [ text "Clear error" ]
        ]

main : Program (Maybe String) Model Msg
main =
  Browser.application
    { view = view
    , init = init
    , update = update
    , subscriptions = always Sub.none
    , onUrlRequest = ClickLink
    , onUrlChange = ChangeUrl
    }

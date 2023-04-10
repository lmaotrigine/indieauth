module Page.Index exposing (view)

import Browser exposing (Document)
import Html exposing (br, h2, p, span, text)
import Layout exposing (basic, template)
import App

type alias Model a =
  { a | tokenData : Maybe App.TokenData }

view : Model a -> Document msg
view { tokenData } =
  case tokenData of
    Nothing ->
      basic "Login Required" []

    Just data ->
      template "Hi"
        ([ h2 [] [ text "Token Info" ]
         , p
            []
            [ span
              []
              [ text "Subscriber: "
              , text data.sub
              , br [] []
              , text "Token ID: "
              , text data.jti
              , br [] []
              , text "Issuer: "
              , text data.iss
              , br [] []
              , text "Audience: "
              , text data.aud
              ]
            ]
          ]
        )

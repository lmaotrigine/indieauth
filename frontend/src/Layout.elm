module Layout exposing (basic, template)

import Browser exposing (Document)
import Html exposing (Html, a, h1, main_, nav, text)
import Html.Attributes exposing (class, href)

basic : String -> List (Html msg) -> Document msg
basic title body =
  { title = title
  , body =
    [ main_
      []
      ([ nav
          [ class "nav" ]
          [ a [ href "/" ] [ text "Hi" ]
          , text " - "
          , a [ href "/login" ] [ text "Login" ]
          ]
        , h1 [] [ text title ]
      ]
        ++ body
      )
    ]
  }

template : String -> List (Html msg) -> Document msg
template title body =
  {  title = title
  , body =
    [ main_
      []
      ([ nav 
          [ class "nav" ]
          [ a [ href "/" ] [ text "Hi" ]
          , text " - "
          ]
        , h1 [] [ text title ]
        ]
          ++ body
      ) 
    ]
  }

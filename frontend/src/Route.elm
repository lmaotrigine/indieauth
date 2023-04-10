module Route exposing (Route(..), routeParser)

import Url.Parser exposing ((</>), Parser, map, oneOf, s)

type Route = Index | Login | NotFound

routeParser: Parser (Route -> a) a
routeParser = oneOf [
  map Index <| s "",
  map Login <| s "login"
  ]

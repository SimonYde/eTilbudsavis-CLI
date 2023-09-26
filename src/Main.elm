module Main exposing (..)

import Browser
import Html exposing (Html, button, div, input, table, text, th, tr)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick)


type alias Offer =
    { name : String
    , dealer : String
    , price : Float
    , cost_per_unit : Float
    , unit : String
    , min_size : Float
    , max_size : Float
    , min_amount : Int
    , max_amount : Int
    , run_from : String
    , run_till : String
    }


type alias Model =
    { search_term : String
    , offers : List Offer
    }


init : Model
init =
    { search_term = ""
    , offers = []
    }


type Msg
    = Search String


view : Model -> Html Msg
view model =
    div []
        [ button [ onClick (Search model.search_term) ] [ text "search" ]
        , input [ placeholder "search term...", value model.search_term ] []
        , table [] <|
            tr []
                [ th [] [ text "Product" ]
                , th [] [ text "Dealer" ]
                , th [] [ text "Price" ]
                , th [] [ text "Cost per Unit" ]
                , th [] [ text "Unit" ]
                , th [] [ text "Minimum size" ]
                , th [] [ text "Maximum size" ]
                , th [] [ text "Minimum purchase amount" ]
                , th [] [ text "Maximum purchase amount" ]
                , th [] [ text "Runs from" ]
                , th [] [ text "Runs until" ]
                ]
                :: List.map
                    viewItem
                    model.offers
        ]


viewItem : Offer -> Html Msg
viewItem offer =
    tr []
        [ th [] [ text offer.name ]
        , th [] [ text offer.dealer ]
        , th [] [ text (String.fromFloat offer.price) ]
        , th [] [ text (String.fromFloat offer.cost_per_unit) ]
        , th [] [ text offer.unit ]
        , th [] [ text (String.fromFloat offer.min_size) ]
        , th [] [ text (String.fromFloat offer.max_size) ]
        , th [] [ text (String.fromInt offer.min_amount) ]
        , th [] [ text (String.fromInt offer.max_amount) ]
        , th [] [ text offer.run_from ]        
        , th [] [ text offer.run_till ]
        ]


update : Msg -> Model -> Model
update msg model =
    case msg of
        Search input ->
            { model | offers = [] }


main : Program () Model Msg
main =
    Browser.sandbox
        { init = init
        , update = update
        , view = view
        }

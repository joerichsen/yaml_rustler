defmodule YamlBenchmark do
  def yaml_sample do
    """
    ---
    receipt:     Oz-Ware Purchase Invoice
    date:        2012-08-06
    customer:
        first_name:   Dorothy
        family_name:  Gale

    items:
        - part_no:   A4786
          descrip:   Water Bucket (Filled)
          price:     1.47
          quantity:  4

        - part_no:   E1628
          descrip:   High Heeled "Ruby" Slippers
          size:      8
          price:     100.00
          quantity:  1

    bill-to:  &id001
        street: |
                123 Tornado Alley
                Suite 16
        city:   East Centerville
        state:  KS

    ship-to:  *id001

    specialDelivery:  >
        Follow the Yellow Brick
        Road to the Emerald City.
        Pay no attention to the
        man behind the curtain.
    """
  end
end

Benchee.run(%{
  "yaml_rustler" => fn -> YamlRustler.parse(YamlBenchmark.yaml_sample()) end,
  "fast_yaml" => fn -> :fast_yaml.decode(YamlBenchmark.yaml_sample()) end
})

from typing import Any

import pulumi


# We're using the Any typehint here since mypy doesn't YET support modules being subtypes of protocols per
# https://github.com/python/mypy/issues/5018 :(
def get_hashicorp_provider_address(
    pulumi_class: Any, provider_type: str, stack: pulumi.StackReference
) -> Any:
    """
    This supports getting a Provider object with an explicit address set.
    This will take the address from the pulumi config file if it is set or fall back to the address in the stack refererence. This allows using SSM port tunneling when run from a workstation.
    :param pulumi_class: Should be one of pulumi_consul or pulumi_nomad
    :param provider_type: One of "consul", "nomad"
    :param stack: The corresponding stack reference
    :return: pulumi.providerReference. Should be used in opts
    """
    override_address = pulumi.Config(provider_type).get("address")
    address = override_address or stack.require_output("address")
    return pulumi_class.Provider(provider_type, address=address)

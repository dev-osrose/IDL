#include "test.h"
#include <iostream>

int main() {
    Packet::Packet packet;
    std::array<char, 32> password{"test"};
    packet.make_request().make_login().set_username("test").set_password(password);
    std::cout << (packet.selection() == Packet::Packet::Selection::REQUEST) << std::endl;
    std::cout << (packet.get_request().selection() == Packet::Request::Selection::LOGIN) << std::endl;
    std::cout << packet.get_request().get_login().get_username().value() << std::endl;
    const auto& pw = packet.get_request().get_login().get_password();
    std::cout << std::string_view(pw.data(), pw.size()) << std::endl;
}
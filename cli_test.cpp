#include "cli_test.h"

using namespace RoseCommon;
using namespace RoseCommon::Packet;



CliTest::CliTest() : CRosePacket(ePacketType::PAKCS_TEST) {}

CliTest::CliTest(CRoseReader reader) : CRosePacket(reader) {
    for (size_t index = 0; index < 42; ++index) {
        if (!reader.get_int(test[index])) {
            return;
        }
    }
}

void CliTest::set_test(const std::array<int, 42>& test) {
    this->test = test;
}

void CliTest::set_test(const int& test, size_t index) {
    this->test[index] = test;
}

const std::array<int, 42>& CliTest::get_test() const {
    return test;
}

const int& CliTest::get_test(size_t index) const {
    return test[index];
}

CliTest CliTest::create(const std::array<int, 42>& test) {
    CliTest packet;
    packet.set_test(test);
    return packet;
}

CliTest CliTest::create(const uint8_t* buffer) {
    CRoseReader reader(buffer, CRosePacket::size(buffer));
    return CliTest(reader);
}

std::unique_ptr<CliTest> CliTest::allocate(const uint8_t* buffer) {
    CRoseReader reader(buffer, CRosePacket::size(buffer));
    return std::make_unique<CliTest>(reader);
}

void CliTest::pack(CRoseBasePolicy& writer) const {
    for (const auto& elem : test) {
        if (!writer.set_int(elem)) {
            return;
        }
    }
}

constexpr size_t CliTest::size() {
    size_t size = 0;
    size += sizeof(int) * 42;
    return size;
}


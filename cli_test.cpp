#include "cli_test.h"

using namespace RoseCommon;
using namespace RoseCommon::Packet;



CliTest::CliTest() : CRosePacket(CliTest::PACKET_ID) {}

CliTest::CliTest(CRoseReader reader) : CRosePacket(reader) {
    if (!reader.get_uint8_t(test)) {
        return;
    }
}

CliTest& CliTest::set_test(const uint8_t test) {
    this->test = test;
    return *this;
}

uint8_t CliTest::get_test() const {
    return test;
}

CliTest CliTest::create(const uint8_t& test) {
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

bool CliTest::pack(CRoseBasePolicy& writer) const {
    if (!writer.set_uint8_t(test)) {
        return false;
    }
    return true;
}

constexpr size_t CliTest::size() {
    size_t size = 0;
    size += sizeof(uint8_t); // test
    return size;
}


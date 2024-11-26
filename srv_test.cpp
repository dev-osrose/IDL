#include "srv_test.h"

using namespace RoseCommon;
using namespace RoseCommon::Packet;

SrvTest::bbb::bbb() : is_valid(false) {}

SrvTest::bbb::bbb(std::string data) : bbb(data), is_valid(false) {
    bool valid = true;
    if (bbb.size() > 3) {
        bbb.resize(3);
        valid &= true;
    } else {
        valid &= true;
    }
    is_valid = valid;
}

bool SrvTest::bbb::read(CRoseReader& reader) {
    bool valid = true;
    if (!reader.get_string(bbb, 3)) {
        return false;
    } else {
        valid &= true;
    }
    is_valid = valid;
    return true;
}

bool SrvTest::bbb::write(CRoseBasePolicy& writer) const {
    if (!writer.set_string(bbb, 3)) {
        return false;
    }
    return true;
}

constexpr size_t SrvTest::bbb::size() {
    size_t size = 0;
    size += 3; // bbb
    return size;
}



SrvTest::SrvTest() : CRosePacket(SrvTest::PACKET_ID) {
    set_server_packet();
}

SrvTest::SrvTest(CRoseReader reader) : CRosePacket(reader) {
    set_server_packet();
    
    if (!reader.get_bitset(bitset1)) {
        return;
    }
    if (!reader.get_uint32_t(c)) {
        return;
    }
    if (!reader.get_bitset(bitset2)) {
        return;
    }
    if (!reader.get_Pote(pote)) {
        return;
    }
    if (!reader.get_Pote2(pote2)) {
        return;
    }
    if (!reader.get_iserialize(x)) {
        return;
    }
    if (!reader.get_iserialize(y)) {
        return;
    }
}

SrvTest& SrvTest::set_a(const uint32_t a) {
    for (size_t i = 0; i < 3; ++i) {
        this->bitset1[i + 0] = a & (1 << i);
    }
    return *this;
}

uint32_t SrvTest::get_a() const {
    uint32_t a_tmp = 0;
    for (size_t i = 0; i < 3; ++i) {
        a_tmp |= (this->bitset1[i + 0] << i);
    }
    return a_tmp;
}

SrvTest& SrvTest::set_b(const uint32_t b) {
    for (size_t i = 0; i < 5; ++i) {
        this->bitset1[i + 3] = b & (1 << i);
    }
    return *this;
}

uint32_t SrvTest::get_b() const {
    uint32_t b_tmp = 0;
    for (size_t i = 0; i < 5; ++i) {
        b_tmp |= (this->bitset1[i + 3] << i);
    }
    return b_tmp;
}

SrvTest& SrvTest::set_c(const uint32_t c) {
    this->c = c;
    return *this;
}

uint32_t SrvTest::get_c() const {
    return c;
}

SrvTest& SrvTest::set_d(const uint32_t d) {
    for (size_t i = 0; i < 1; ++i) {
        this->bitset2[i + 0] = d & (1 << i);
    }
    return *this;
}

uint32_t SrvTest::get_d() const {
    uint32_t d_tmp = 0;
    for (size_t i = 0; i < 1; ++i) {
        d_tmp |= (this->bitset2[i + 0] << i);
    }
    return d_tmp;
}

SrvTest& SrvTest::set_e(const uint32_t e) {
    for (size_t i = 0; i < 14; ++i) {
        this->bitset2[i + 1] = e & (1 << i);
    }
    return *this;
}

uint32_t SrvTest::get_e() const {
    uint32_t e_tmp = 0;
    for (size_t i = 0; i < 14; ++i) {
        e_tmp |= (this->bitset2[i + 1] << i);
    }
    return e_tmp;
}

SrvTest& SrvTest::set_f(const uint32_t f) {
    for (size_t i = 0; i < 1; ++i) {
        this->bitset2[i + 15] = f & (1 << i);
    }
    return *this;
}

uint32_t SrvTest::get_f() const {
    uint32_t f_tmp = 0;
    for (size_t i = 0; i < 1; ++i) {
        f_tmp |= (this->bitset2[i + 15] << i);
    }
    return f_tmp;
}

SrvTest& SrvTest::set_g(const uint32_t g) {
    for (size_t i = 0; i < 31; ++i) {
        this->bitset2[i + 16] = g & (1 << i);
    }
    return *this;
}

uint32_t SrvTest::get_g() const {
    uint32_t g_tmp = 0;
    for (size_t i = 0; i < 31; ++i) {
        g_tmp |= (this->bitset2[i + 16] << i);
    }
    return g_tmp;
}

SrvTest& SrvTest::set_h(const uint32_t h) {
    for (size_t i = 0; i < 1; ++i) {
        this->bitset2[i + 47] = h & (1 << i);
    }
    return *this;
}

uint32_t SrvTest::get_h() const {
    uint32_t h_tmp = 0;
    for (size_t i = 0; i < 1; ++i) {
        h_tmp |= (this->bitset2[i + 47] << i);
    }
    return h_tmp;
}

SrvTest& SrvTest::set_pote(const SrvTest::Pote pote) {
    this->pote = pote;
    return *this;
}

SrvTest::Pote SrvTest::get_pote() const {
    return pote;
}

SrvTest& SrvTest::set_pote2(const SrvTest::Pote2 pote2) {
    this->pote2 = pote2;
    return *this;
}

SrvTest::Pote2 SrvTest::get_pote2() const {
    return pote2;
}

SrvTest& SrvTest::set_x(const Aaa x) {
    this->x = x;
    return *this;
}

Aaa SrvTest::get_x() const {
    return x;
}

SrvTest& SrvTest::set_y(const Bbb y) {
    this->y = y;
    return *this;
}

Bbb SrvTest::get_y() const {
    return y;
}

SrvTest SrvTest::create(const uint32_t& a, const uint32_t& b, const uint32_t& c, const uint32_t& d, const uint32_t& e, const uint32_t& f, const uint32_t& g, const uint32_t& h, const SrvTest::Pote& pote, const SrvTest::Pote2& pote2, const Aaa& x, const Bbb& y) {
    SrvTest packet;
    packet.set_a(a);
    packet.set_b(b);
    packet.set_c(c);
    packet.set_d(d);
    packet.set_e(e);
    packet.set_f(f);
    packet.set_g(g);
    packet.set_h(h);
    packet.set_pote(pote);
    packet.set_pote2(pote2);
    packet.set_x(x);
    packet.set_y(y);
    return packet;
}

SrvTest SrvTest::create(const uint8_t* buffer) {
    CRoseReader reader(buffer, CRosePacket::size(buffer));
    return SrvTest(reader);
}

std::unique_ptr<SrvTest> SrvTest::allocate(const uint8_t* buffer) {
    CRoseReader reader(buffer, CRosePacket::size(buffer));
    return std::make_unique<SrvTest>(reader);
}

bool SrvTest::pack(CRoseBasePolicy& writer) const {
    if (!writer.set_bitset(bitset1)) {
        return false;
    }
    if (!writer.set_uint32_t(c)) {
        return false;
    }
    if (!writer.set_bitset(bitset2)) {
        return false;
    }
    if (!writer.set_Pote(pote)) {
        return false;
    }
    if (!writer.set_Pote2(pote2)) {
        return false;
    }
    if (!writer.set_iserialize(x)) {
        return false;
    }
    if (!writer.set_iserialize(y)) {
        return false;
    }
    return true;
}

constexpr size_t SrvTest::size() {
    size_t size = 0;
    size += 8 / 8; // bitset1
    size += sizeof(uint32_t); // c
    size += 48 / 8; // bitset2
    size += sizeof(Pote); // pote
    size += sizeof(Pote2); // pote2
    size += sizeof(Aaa); // x
    size += sizeof(Bbb); // y
    return size;
}


void RoseCommon::Packet::to_json(nlohmann::json& j, const SrvTest::aaa& data) {
    j = nlohmann::json{
        { "value", static_cast<uint8_t>(data) },
    };
}
void RoseCommon::Packet::to_json(nlohmann::json& j, const SrvTest::bbb& data) {
    j = nlohmann::json{
        { "value", data.operator std::string() },
    };
}

void RoseCommon::Packet::to_json(nlohmann::json& j, const SrvTest& data) {
    j = nlohmann::json{
        { "metadata", { { "packet", "PAKWC_TEST" }, { "size", data.get_size() } } },
        { "fields", {
            { "a", data.get_a() == 1 },
            { "b", data.get_b() == 1 },
            { "c", data.get_c() },
            { "d", data.get_d() == 1 },
            { "e", data.get_e() == 1 },
            { "f", data.get_f() == 1 },
            { "g", data.get_g() == 1 },
            { "h", data.get_h() == 1 },
            { "pote", data.get_pote() },
            { "pote2", data.get_pote2() },
            { "x", data.get_x() },
            { "y", data.get_y() },
        } }
    };
}

